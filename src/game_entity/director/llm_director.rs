use std::{collections::{HashMap, HashSet}, sync::LazyLock};

use hord3::horde::{game_engine::world::WorldComputeHandler, geometry::vec3d::Vec3D};
use html_parser::{Dom, Node};
use proxima_backend::{ai_interaction::endpoint_api::EndpointRequestVariant, database::{chats::SessionType, configuration::{ChatConfiguration, ChatSetting}, context::{ContextData, ContextPart, ContextPosition, WholeContext}}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{game_engine::{CoolGameEngineTID, CoolVoxel}, game_entity::{actions::{Action, ActionCounter, ActionKind, ActionSource, ActionTimer, ActionsEvent, ActionsUpdate}, director::{DirectorAlert, DirectorEvent}, GameEntityEvent, GameEntityVecRead}, game_map::{get_voxel_pos, GameMap, Voxel, VoxelLight, WorldVoxelPos}, proxima_link::{HordeProximaAIRequest, HordeProximaAIResponse}};

static PATHING_POSITIONS:LazyLock<Vec<Vec3D<i32>>> = LazyLock::new(|| {
    vec![
        Vec3D::new(1, 0, 0),
        Vec3D::new(0, 1, 0),
        Vec3D::new(-1, 0, 0),
        Vec3D::new(0, -1, 0),
        Vec3D::new(1, 1, 0),
        Vec3D::new(1, -1, 0),
        Vec3D::new(-1, 1, 0),
        Vec3D::new(-1, -1, 0),

        Vec3D::new(0, 0, -1),
        Vec3D::new(1, 0, -1),
        Vec3D::new(0, 1, -1),
        Vec3D::new(-1, 0, -1),
        Vec3D::new(0, -1, -1),
        Vec3D::new(1, 1, -1),
        Vec3D::new(1, -1, -1),
        Vec3D::new(-1, 1, -1),
        Vec3D::new(-1, -1, -1),

        Vec3D::new(0, 0, 1),
        Vec3D::new(1, 0, 1),
        Vec3D::new(0, 1, 1),
        Vec3D::new(-1, 0, 1),
        Vec3D::new(0, -1, 1),
        Vec3D::new(1, 1, 1),
        Vec3D::new(1, -1, 1),
        Vec3D::new(-1, 1, 1),
        Vec3D::new(-1, -1, 1),
    ]
});

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct LLMDirector {
    pub in_flight_prompts:HashMap<usize, (usize, CoolGameEngineTID, Vec<DirectorAlert>, Vec<String>)>,
    pub latest_id:usize,
    pub responses:Vec<HordeProximaAIResponse>,
    goals:Vec<String>,
    memory:Vec<String>,
    pub feedback:Vec<String>,
    last_prompt_tick:usize,
}

pub fn get_world_slice_string<'a>(
    from:WorldVoxelPos,
    to:WorldVoxelPos, 
    agent_id:usize,
    first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
    second_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
    world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>
) -> String {
    let mut final_string = String::new();
    let z = from.z;
    final_string += &format!("<map z = {}>\n   ", z);
    for x in from.x..=to.x {
        let string = format!("{}", x);
        // This works because numbers are represented entirely in ASCII here
        // at least I assume they are
        if string.len() == 1 {
            final_string += &format!("  {}  ", string);
        }
        else if string.len() == 2 {
            final_string += &format!(" {}  ", string);
        }
        else if string.len() == 3 {
            final_string += &format!(" {} ", string);
        }
        else if string.len() == 4 {
            final_string += &format!("{} ", string);
        }
        else {
            final_string += &format!("     ");
        }
    }
    let mut render_agents:HashMap<(i32,i32), usize> = HashMap::with_capacity(16);
    for (i, ent) in first_ent.movement.iter().enumerate() {
        if i != agent_id {
            let pos = get_voxel_pos(ent.pos);
            if pos.z == z && pos.x >= from.x && pos.x < to.x && pos.y >= from.y && pos.y < to.y {
                render_agents.insert((pos.x, pos.y), i);
            }
        }
    }
    let agent_pos = get_voxel_pos(first_ent.movement[agent_id].pos);
    final_string += &format!("\n");
    for y in from.y..=to.y {
        let string = format!("{}", y);
        if string.len() == 1 {
            final_string += &format!(" {} ", string);
        }
        else if string.len() == 2 {
            final_string += &format!("{} ", string);
        }
        else if string.len() == 3 {
            final_string += &format!("{}", string);
        }
        else {
            final_string += &format!("   ");
        }
        for x in from.x..=to.x {
            if Vec3D::new(x, y, z) == agent_pos {
                final_string += &format!("  @  ");
            }
            else {
                match render_agents.get(&(x,y)) {
                    Some(other_agent_id) => final_string += &format!("  a  "),
                    None => 
                    {
                        if world.world.is_voxel_solid(Vec3D::new(x, y, z)) {
                            if let Some(voxel) = world.world.get_voxel_at(Vec3D::new(x, y, z)) && voxel.voxel_id() == 9 {
                                final_string += &format!("  µ  ");
                            }
                            else {
                                final_string += &format!("  %  ");
                            }
                            
                        }
                        else if world.world.is_voxel_solid(Vec3D::new(x, y, z - 1)) {
                            final_string += &format!("  -  ");
                        }
                        else {
                            final_string += &format!("  .  ");
                        }
                    }
                    
                }
            }
        }

        final_string += &format!("\n");
    }

    if render_agents.len() > 0 {
        final_string += &format!("\nother agents :\n");
        for ((x,y), other_id) in render_agents {
            final_string += &format!("  - ({x}, {y}, {z}) : {}\n", first_ent.director[other_id].get_name());
        }
    }

    final_string += &format!("\n</map z = {}>\n", z);
    final_string
}

impl LLMDirector {
    pub fn new_with_goals(goals:Vec<String>) -> Self {
        Self { in_flight_prompts: HashMap::with_capacity(4), latest_id: 0, responses: Vec::new(), goals, last_prompt_tick: 0, memory:Vec::with_capacity(4), feedback:Vec::new() }
    }
    pub fn get_periodic_payload<'a>(
        &mut self,
        agent_id:usize,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>,
        tick:usize,
    ) -> Option<HordeProximaAIRequest> {
        if tick - self.last_prompt_tick > 1000 && self.in_flight_prompts.len() == 0 {
            Some(self.get_payload(agent_id, vec![DirectorAlert::Periodic], first_ent, second_ent, world, tick))
        }
        else {
            None
        }
    }
    pub fn get_payload<'a>(
        &mut self,
        agent_id:usize,
        reasons:Vec<DirectorAlert>,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>,
        tick:usize,
    ) -> HordeProximaAIRequest {
        const SIGHT_RANGE:i32 = 4;

        let mut total_context = WholeContext::new(vec![]);

        let mut system = String::from(include_str!("system_prompt.txt"));
        //system = system.replace("AGENT_NAME", &first_ent.director[agent_id].get_name());

        total_context.add_part(ContextPart::new(vec![ContextData::Text(system)], ContextPosition::System));

        let mut specific_prompt = String::new();

        specific_prompt += &format!("<input_data>\n\n");

        let agent_pos = get_voxel_pos(first_ent.movement[agent_id].pos);

        specific_prompt += &format!("<personal_info>\nname:{}\nposition : ({}, {}, {}) \n</personal_info>\n",&first_ent.director[agent_id].get_name(), agent_pos.x, agent_pos.y, agent_pos.z);


        specific_prompt += &format!("<memory>\n");

        for (i, goal) in self.memory.iter().enumerate() {
            specific_prompt += &format!("{}. {}\n",i+1, goal);
        }

        specific_prompt += &format!("</memory>\n");

        if self.feedback.len() > 0 {
            specific_prompt += &format!("<feedback>\n");

            for (i, goal) in self.feedback.iter().enumerate() {
                specific_prompt += &format!("- {}\n", goal);
            }

            specific_prompt += &format!("</feedback>\n");
        }


        for z in (agent_pos.z - 1)..=(agent_pos.z + 1) {
            specific_prompt += &get_world_slice_string(Vec3D::new(agent_pos.x - SIGHT_RANGE, agent_pos.y - SIGHT_RANGE, z), Vec3D::new(agent_pos.x + SIGHT_RANGE, agent_pos.y + SIGHT_RANGE, z), agent_id, first_ent, second_ent, world);

        }



        specific_prompt += &format!("<goals>\n");

        for (i, goal) in self.goals.iter().enumerate() {
            specific_prompt += &format!("{}. {}\n",i+1, goal);
        }

        specific_prompt += &format!("</goals>\n<prompt_reason>\n");
        for reason in &reasons {
            match reason {
                DirectorAlert::Periodic => specific_prompt += &format!("Periodic prompting after inactivity\n"),
                DirectorAlert::HeardWords(from, text) => specific_prompt += &format!("You have heard the following from {} : \"{}\"\n", first_ent.director[*from].get_name(), text),
                DirectorAlert::FinishedMoveTo(to, worked) => if *worked {
                    specific_prompt += &format!("Finished moving to ({}, {}, {})\n", to.x, to.y, to.z);
                }
                else {
                    specific_prompt += &format!("Couldn't move to ({}, {}, {})\n", to.x, to.y, to.z);
                }
            }
        }
        

        specific_prompt += &format!("</prompt_reason>\n\n</input_data>\n");

        total_context.add_part(ContextPart::new(vec![ContextData::Text(specific_prompt)], ContextPosition::User));

        let request_id = self.latest_id;
        self.latest_id += 1;

        self.in_flight_prompts.insert(request_id, (request_id, CoolGameEngineTID::entity_1(agent_id), reasons, self.feedback.clone()));

        self.last_prompt_tick = tick;

        HordeProximaAIRequest::new(
            request_id, CoolGameEngineTID::entity_1(agent_id), 
            EndpointRequestVariant::RespondToFullPrompt { 
                whole_context: total_context,
                streaming: false,
                session_type: SessionType::Chat,
                chat_settings: Some(ChatConfiguration::new(String::from("config_config"),
                vec![
                    ChatSetting::ResponseTokenLimit(3000),
                    ChatSetting::Temperature(90)
                ])) 
            }
        )

        
    }
    pub fn parse_responses<'a>(
        &mut self,
        agent_id:usize,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>,
        tick:usize,
        counter:&mut ActionCounter,
    ) {
        for response in self.responses.clone() {
            self.parse_response(response.clone(), agent_id, first_ent, second_ent, world, tick, counter);
        }
        self.responses.clear();
    }
    fn parse_response<'a>(
        &mut self,
        response:HordeProximaAIResponse,
        agent_id:usize,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>,
        tick:usize,
        counter:&mut ActionCounter,
    ) {
        if self.in_flight_prompts.remove(&response.request_id).is_some() {
            match response.response {
                Some(text) => {
                    match Dom::parse(&text) {
                        Ok(parsed) => {
                            for child in parsed.children {
                                match child {
                                    Node::Element(elt) => {
                                        match elt.name.trim() {
                                            "actions" => {
                                                if elt.children.len() > 0 && let Some(text) = elt.children[0].text() {
                                                    self.parse_commands(String::from(text), agent_id, first_ent, second_ent, world, tick, counter);
                                                }
                                                break;
                                            },
                                            _ => ()
                                        }
                                    },
                                    _ => ()
                                }
                            }
                        },
                        Err(_) => ()
                    }
                },
                None => ()
            }
        }
    }
    fn parse_commands<'a>(
        &mut self,
        commands:String,
        agent_id:usize,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>,
        tick:usize,
        counter:&mut ActionCounter,
    ) {
        let mut remove_goals = Vec::with_capacity(5);
        let mut remove_memory = Vec::with_capacity(5);
        for line in commands.lines() {
            let words:Vec<&str> = line.split_whitespace().collect();
            let others:Vec<&str> = line.split('"').collect();
            if words.len() >= 3 {
                match words[0] {
                    "SAY" => match words[1] {
                        "closest" => if others.len() > 1 {
                            let text = String::from(others[1]);
                            let mut closest:Option<(usize, f32)> = None;
                            let agent_pos = first_ent.movement[agent_id].pos;
                            for i in 0..first_ent.director.len() {
                                if i != agent_id {
                                    match closest {
                                        Some((id, dist)) => {
                                            let new_dist = agent_pos.dist(&first_ent.movement[i].pos);
                                            if new_dist < dist {
                                                closest = Some((i, new_dist));
                                            }
                                        },
                                        None => {closest = Some((i, agent_pos.dist(&first_ent.movement[i].pos)));}
                                    }
                                }
                            }
                            match closest {
                                Some((id, dist)) => {first_ent.tunnels.director_out.send(GameEntityEvent::new(true,DirectorEvent::new(id, Some(CoolGameEngineTID::entity_1(agent_id)), super::DirectorUpdate::SendAlert(DirectorAlert::HeardWords(agent_id, text)))));},
                                None => ()
                            }
                        },
                        "local" => {
                            let text = String::from(others[1]);
                            let agent_pos = first_ent.movement[agent_id].pos;
                            let mut total_count = 0;
                            for i in 0..first_ent.director.len() {
                                if i != agent_id {
                                    let dist = agent_pos.dist(&first_ent.movement[i].pos);
                                    if dist < 10.0 {
                                        first_ent.tunnels.director_out.send(GameEntityEvent::new(true,DirectorEvent::new(i, Some(CoolGameEngineTID::entity_1(agent_id)), super::DirectorUpdate::SendAlert(DirectorAlert::HeardWords(agent_id, text.clone())))));
                                        total_count += 1;
                                    }
                                }
                            }
                            if total_count > 0 {
                                self.feedback.push(format!("Your last SAY local command reached {} other agents", total_count));
                            }   
                            else {
                                self.feedback.push(format!("Your last SAY local command reached {} other agents, consider using the \"closest\" or \"all\" modes to be certain to reach someone", total_count));
                            }
                        },
                        "all" => {
                            let text = String::from(others[1]);
                            let mut total_count = 0;
                            for i in 0..first_ent.director.len() {
                                if i != agent_id {
                                    first_ent.tunnels.director_out.send(GameEntityEvent::new(true,DirectorEvent::new(i, Some(CoolGameEngineTID::entity_1(agent_id)), super::DirectorUpdate::SendAlert(DirectorAlert::HeardWords(agent_id, text.clone())))));
                                    total_count += 1;
                                }
                            }
                            self.feedback.push(format!("Your last SAY all command reached {} other agents", total_count));
                        },
                        _ => ()
                    },
                    "GOTO" => if words.len() == 4 {
                        if let Ok(x) = words[1].parse::<i32>() && let Ok(y) = words[2].parse::<i32>() && let Ok(z) = words[3].parse::<i32>() {
                            let id = counter.get_next_id();
                            let mut final_position = Vec3D::new(x, y, z);

                            if world.world.is_voxel_solid(final_position) || !world.world.is_voxel_solid(final_position - Vec3D::new(0, 0, 1)) {
                                for pos in PATHING_POSITIONS.iter() {
                                    let test_pos = final_position + pos;
                                    if !world.world.is_voxel_solid(test_pos) && world.world.is_voxel_solid(test_pos - Vec3D::new(0, 0, 1)) {
                                        final_position = test_pos;
                                        break;
                                    }
                                }
                            }
                            first_ent.tunnels.actions_out.send(GameEntityEvent::new(true,ActionsEvent::new(agent_id, None, ActionsUpdate::AddAction(Action::new(id, tick, ActionTimer::Delay(15000), ActionKind::PathToPosition(Vec3D::new(final_position.x as f32, final_position.y as f32, final_position.z as f32), 0.8), ActionSource::Director)))));
                        }
                    },
                    "GOAL" => match words[1] {
                        "add" => {
                            if others.len() > 1 {
                                self.goals.push(String::from(others[1]));
                            }
                        },
                        "remove" => if let Ok(number) = words[2].parse::<usize>() {
                            match number.checked_add_signed(-1) {
                                Some(real_number) => if self.goals.len() > real_number && !remove_goals.contains(&real_number) {
                                    remove_goals.push(real_number);
                                },
                                None => ()
                            }
                        },
                        _ => ()
                    },
                    "MEMORY" => match words[1] {
                        "add" => {
                            if others.len() > 1 {
                                self.memory.push(String::from(others[1]));
                            }
                        },
                        "remove" => if let Ok(number) = words[2].parse::<usize>() {
                            match number.checked_add_signed(-1) {
                                Some(real_number) => if self.memory.len() > real_number && !remove_memory.contains(&real_number) {
                                    remove_memory.push(real_number);
                                },
                                None => ()
                            }
                        },
                        _ => ()
                    },
                    "BLOCK" => if words.len() == 5 {
                        match words[1] {
                            "place" => if let Ok(x) = words[2].parse::<i32>() && let Ok(y) = words[3].parse::<i32>() && let Ok(z) = words[4].parse::<i32>() {
                                let id = counter.get_next_id();
                                first_ent.tunnels.actions_out.send(GameEntityEvent::new(true,ActionsEvent::new(agent_id, None, ActionsUpdate::AddAction(Action::new(id, tick, ActionTimer::Delay(500), ActionKind::ChangeVoxel(Vec3D::new(x, y, z), CoolVoxel::new(9, 0, VoxelLight::max_light(), None)), ActionSource::Director)))));
                            },
                            "destroy" => if let Ok(x) = words[2].parse::<i32>() && let Ok(y) = words[3].parse::<i32>() && let Ok(z) = words[4].parse::<i32>() {
                                let id = counter.get_next_id();
                                first_ent.tunnels.actions_out.send(GameEntityEvent::new(true,ActionsEvent::new(agent_id, None, ActionsUpdate::AddAction(Action::new(id, tick, ActionTimer::Delay(500), ActionKind::ChangeVoxel(Vec3D::new(x, y, z), CoolVoxel::new(0, 0, VoxelLight::max_light(), None)), ActionSource::Director)))));
                            },
                            _ => ()
                        }
                    },
                    "FILL" => if words.len() == 8 {
                        match words[1] {
                            "place" => if let Ok(mut x1) = words[2].parse::<i32>() && let Ok(mut y1) = words[3].parse::<i32>() && let Ok(mut z1) = words[4].parse::<i32>() && let Ok(mut dx) = words[5].parse::<i32>() && let Ok(mut dy) = words[6].parse::<i32>() && let Ok(mut dz) = words[7].parse::<i32>() {
                                let (x2, y2, z2) = (x1 + dx, y1 + dy, z1 + dz);
                                let delay = ActionTimer::Delay((dx * dy * dz) as usize + 400);
                                for x in x1..x2 {
                                    for y in y1..y2 {
                                        for z in z1..z2 {
                                            let id = counter.get_next_id();
                                            first_ent.tunnels.actions_out.send(GameEntityEvent::new(true,ActionsEvent::new(agent_id, None, ActionsUpdate::AddAction(Action::new(id, tick, delay.clone(), ActionKind::ChangeVoxel(Vec3D::new(x, y, z), CoolVoxel::new(9, 0, VoxelLight::max_light(), None)), ActionSource::Director)))));
                                        }
                                    }
                                }
                            },
                            "destroy" => if let Ok(mut x1) = words[2].parse::<i32>() && let Ok(mut y1) = words[3].parse::<i32>() && let Ok(mut z1) = words[4].parse::<i32>() && let Ok(mut dx) = words[5].parse::<i32>() && let Ok(mut dy) = words[6].parse::<i32>() && let Ok(mut dz) = words[7].parse::<i32>() {
                                let (x2, y2, z2) = (x1 + dx, y1 + dy, z1 + dz);
                                let delay = ActionTimer::Delay((dx * dy * dz) as usize + 400);
                                for x in x1..x2 {
                                    for y in y1..y2 {
                                        for z in z1..z2 {
                                            let id = counter.get_next_id();
                                            first_ent.tunnels.actions_out.send(GameEntityEvent::new(true,ActionsEvent::new(agent_id, None, ActionsUpdate::AddAction(Action::new(id, tick, delay.clone(), ActionKind::ChangeVoxel(Vec3D::new(x, y, z), CoolVoxel::new(0, 0, VoxelLight::max_light(), None)), ActionSource::Director)))));
                                        }
                                    }
                                }
                            },
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
        }
        remove_goals.sort();
        for number in remove_goals.iter().rev() {
            self.goals.remove(*number);
        }
        remove_memory.sort();
        for number in remove_memory.iter().rev() {
            self.memory.remove(*number);
        }
    }
}



