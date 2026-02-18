# Idée générale

Horde shooter avec du multi

carte carrée avec un objectif à défendre au centre

X minutes de préparation où on peut poser des murs et tout

3 niveaux de mur :
 - cassables
 - cassables mais se répare automatiquement à chaque vague
 - très difficiles à casser et se répare automatiquement à chaque vague

blocs piège :
 - DOT
    - piques
    - magma
    - électrique
 - tourelle
    - standard
    - rapide
    - perce 



   
# Comment faire du multijoueur

- TID = total ID, indique si c'est le monde, ou ent1, ou ent2
- INT = intéraction, contient un event
- le serveur garde une `HashMap<TID, Vec<INT>>`, la "intermap" pour chaque interaction
- nombre de tps fixe
- chaque packet est marqué de quel tick il vient
- générateurs de random synchronisés entre serveur/clients pour générer des ID
   - chaque ID sert à voir si les hash des champs exacts sont identiques, et si les champs à incertitudes sont proches
   - si y'a une différence, le serveur renvoie tout ce qui doit être synchronisé pour les N derniers ticks avec les intermap correspondantes
- toujours tester les entités de joueurs aussi
- tag d'event dans le protocole pour dire que ça vient d'un joueur (donc à passer à tout le monde)

# concepts plus précis

## Dungeon Crawler 9000

- équipe de 4
- donjon généré aléatoirement (peut être des voxels)
- étage par étage (les étages font pas forcément sens thématiquement ala backrooms)
- chaque salle doit être finie avant d'ouvrir la porte à une autre
- minimap dans l'UI
- level up d'équipe avec de l'XP par salle
- 




## Comment ça marche les voxels

- empty : 0b00XXXXXX
- pour tourner, on applique d'abord le 00, puis le 000
   - application du 00
      - LUT pour savoir où chaque bit du empty se swap
   - application du 000

## How to do voxel collision
- do other kinds of collision first in main tick and add to speed
- do voxel checks in after main
- if any point of the AABB is inside terrain :
   - compute the "speed nudge" needed to get it to an empty tile quickest in all directions
   - compare all nudges, take the smallest one in all directions (in absolute terms) that makes the collider no longer collide with anything
   - give priority to up/down, if only an up/down nudge works, do that

## Experiment concept : Agent playground
- every entity is an "agent"
- agents have :
   - collider
   - inventory
   - movement characteristics
      - jump height
      - movement speed
   - "Planning AI"
      - pathfinding/moving through path
         - A* through integer grid
         - start at integered origin, explore in integer direction towards the objective
         - if that has already been explored, explore in all directions until you can explore towards the objective
         - have HashSet of all positions that have been explored (as in a path already goes through there, or it's impassable)
         - make a tree inside a vec
            - each node has a parent, tried directions (with their node ids) and 
      - aiming/shooting
      - keeps track of 
   - "Actions"
      - list of "actions" like :
         - jumping
         - moving in a direction
         - turning
         - placing/breaking block
         - using an item
      - if an action can be done/tried directly, do it (e.g. jumping)
      - if it requires planning, ask "planning AI" for extra actions
      - each action has an ID, so the Director can know when a multi-step action has ended
      - an action can be given a deadline/timeout (in game ticks)
      - an action can either :
         - End (was possible to try, and tried)
         - timeout (wasn't done in time)
   - "Director"
      - provides actions to perform
      - can be a player, an LLM, an expert system, nothing, a loop...
      - LLM Director
         - how to provide world information to an LLM ?
            - grid of nearby tiles for sight, à la old roguelikes
            - (x,y) grid where x are numbers and y are letters
            - 3 grids provided, a layer below, the entity's layer, and a layer above (layers are numbered -1, 0, 1)
            - characters used for world description :
               - `,` : solid ground (can stand there)
               - `.` : empty (would fall there)
               - `%` : solid block on same level (can't go through that)
               - `a` : other agent on same level
               - `@` : agent
            - describe who each other agent is by (coordinates) : name
         - how can an LLM act ?
            - actions as function calls :
               - format : same as Proxima (needs to be evolved so it works better)
               - possible actions :
                  - SAY {text}
                  - 
         - how to connect LLMs to the engine ?
            - agents have prompt response vecs and request ID generators
            - extra_data contains a Sender<HordeProximaRequest>
            - HordeProximaRequest :
               - request id
               - source entity (TID)
               - Prompt
            - another thread handles talking to Proxima
               - that thread has the Receiver<HordeProximaRequest>
               - has a Sender<HordeProximaResponse>
                  - contains the TID and request ID, as well as the response
            - the main thread (or any tick-synchronized thread) has the Receiver<HordeProximaResponse>
               - each tick, receives and creates events filling the prompt vecs of the relevant entities


# Fixing multiplayer performance
- what could be causing issues ?
- huge function stacks for each byte
- solution :
   - decode_bytes_borrow but generalized

- 2 new issues :
   - data rates (sending too much data in peak times)
      - for agrees, send only hash of what you want to check
      - only check
   - too much TCP desync :
      - server listens at t1, client sends at t2, listens at t3, server sends at t4
      - this causes lots of TCP errors as the server may fill buffers and cause window errors
      - we need streams to always listen and write for both clients and server
      - new multiplayer architecture :
         - spin up 1 thread per client (bad ?)
         - alternate between read + decode and encode + write
            - encoding can be done from another thread that sends the event to write for example to reduce read downtime to a minimum, same for decodes (though that would require unecessary allocation)
            - those threads are seperated from the orchestrator

- how to fix all that
   - make reads timeout on tickrate/2 to reduce syscalls


# Game concept : racing battle royale

- multiplayer racing battle royale
- randomly generated map that is revealed as the players drive on it
- at the start the road is wide and with few/no obstacles
- the zone is a road distance to the first place driver, influenced by how far ahead they are
- start in medias res, air dropped at low but non-zero speeds on the start line
- "speed stages"
   - everyone starts with 0 nitro, only pure speed
   - as we approach the end of stage 0, everyone gets access to nitro, from the people in the back first
   - there are still speed boosts randomly strewn across the road
   - by the end of stage 0, everyone has access to nitro
   - each subsequent stage brings top speed to the speed of the nitro at the start of the previous stage, and nitro is boosted but less fast than the speed increases
      - makes mistakes more punishing as you go to end the game fastest in the end game
- different vehicles
   - trucks
   - cars
   - motorcycles
   - all vehicles should have the same top speed, the main differences would be health/weight/maneuverability/acceleration
      - health :
         - lower health = lower top speed, recover health in bursts with repair kits found on the road or slowly over time when not attacked
         - getting hit by weapons and hitting obstacles (static or moving) = losing health
      - weight :
         - more weight = slower health regen
         - the vehicle with more weight is affected less by a collision than the one with less
      - maneuverability
      - acceleration

- different drivers
   - each driver will have an ability unrelated to speed
   - e.g. :
      - translating across the road dash-style
      - jumping
      - temporary directional shield
      - rotating X degrees without losing speed
      - temporary hover
      - fireball
   - all drivers will have a base weapon
      - all weapons will be fairly inaccurate
         - only a hail of bullets from people behind first place should work to slow down first place
      - no timed dropped explosives, would be too favorable to people further up the field
      - weapons will only be shot from the driver's seat
         - makes huge trucks unable to handle bikes that get dangerously close
   - drivers I want to have :
      - wizard/witch
      - knight in armor
      - person in shorts and tshirt
      - gorilla
      - cat with a propeller hat
      - pirate
      - sweet JP
      - speed racer

- stage themes
   - basically the kind of planet the stage happens on, also announced before the game starts
   - themes include :
      - city (with different densities through the stage)
         - very dense would be basically hive cities
         - neon lighting because of course
      - nature
         - lush
         - desert
         - dead
      - caverns

- stage hazards
   - before the game starts, the hazards are decided and communicated to players as they choose driver/vehicle
   - hazards include :
      - temporary fog
         - makes players drive from the minimap and distance sensors
         - abilities could make you see through it better
      - meteor shower
         - helldivers-style, but leaves obstacles (that may act as ramps !!!)
      - volcanic activity
         - replaces water on the stage with lava
      - earthquakes
      - short day/night cycle
      - strong winds/tornadoes
      - police
      - wizards

# How do you make vehicles :
- pankek

- hull, 3D collider mesh
   - it will be 1 solid piece to simplify everything else

- specify all locomotion equipment in entity-local coordinates
   - wheels
   - thrusters
   - hover
- all that equipment has to have :
   - collider
   - activation requirement(s) :
      - nitro amount
      - contact with surface
      - distance to surface
      - action from the driver
   - motion vector when activated
      - equipment-local coordinates, where it "pushes" you
      - should it have accurate physics (lever) or not
      - boolean or analog
   - can it turn
      - does it turn physically or does its motion vector just change
      - limits of turning
   - eventual local movement due to external sources/etc
      - suspension for wheels
      - friction
   - can it be affected by nitro ?
- aerodynamics ?
   - no
   - nuh huh

- best overall architecture :
   - drivers as the first entities
   - vehicles as the second entities
   - that way you can just compute vehicle/vehicle collisions


# How do you make vehicle/ground collisions

- first : AABB
- if AABB is crossed, use more complex collider
- check hull next :
   - for each vertex making up the hull :


# How do you make motion line / polygonal face collisions with pushback
- get line/plan collision

# How to make locomotion happen
- first part of the tick :
   - check if any locomotion equipment can activate [x]
      - to check vehicle stats (nitro,etc) => need vehicle [x]
      - to check surface contact/distance => need world and raycasting [x]
      - to check driver stats (actions, etc) => need driver [x]
   - if it can :
      - compute the final motion vector [x]
      - compute how much of that vector must be applied [x]
      - get the center of gravity and application point [x]
      - bras de levier et tout (jsp comment on fait mtn) [x]
      - implement pidget over ip (?)
   - add gravity to speed in any case
   - check collision with other vechicles there
- second part of the tick :
   - check locomotion equipment collision with the world
   - rough check -> get local pos shifted to world pos, rotate equipment down, raycast until you hit the world and check if it's within bounding collider
      - store if touching the ground and which ground it's touching, and if not the distance to the ground
   - get necessary pushback from this 
   - check hull collision with the world

# how to improve multiplayer AGAIN

- seperate must_sync bool into enum
   - MustSync
      - No
      - Server
      - Client
      - ClientIfReservedID
   - stuff is only synced accordingly, e.g. actions are client synced if reserved ID, positions are server synced

- DONE

# how to do road generation

- choose start chunk pos and direction
- every "step", advance a random amount of steps in the desired direction and change direction
- store all affected chunks, generate them

# how to optimize terrain LOD generation
- compute which directions you actually need
- if empty, empty lods
- if not empty
   - if full of one type 
      - look at adjacent chunks
      - if also full of one full type, remove that direction from further compute
      - otherwise, keep that direction

# improvements
- make local turn speed decrease over time and stop when turn is at limits
- make normal speed follow ground locomotion direction :)
   - divide speed vector between all ground locomotion equipment currently touching ground
   - compute local motion vectors and all that for all of them

- apply gravity to each locomotion equipment seperately when not touching ground

- handle disconnects gracefully
   - change decode_from_tcp to Result<Vec<T>>

- synchronize ticks
- even better multiplayer sync
- make 

- make movement vectors configurable dynamically
   - along flat surface

# new high speed collision model

- compute collision from on_ground locomotion equipment
   - current position in world vs next position in world
   - draw vector/arc between both
   - test discrete steps between the 2, first that has a voxel, compute nearest nudge, and vehicle reaction to it
      - how to make sure the nudge keeps the car out of the ground ?
      - other idea :
      - first that has a voxel, compute the speed diff starting from collision point that needs to be cancelled (z first)
      - have to compute every equipment on ground first to divide vector between them
   - if no voxel in path, compute gravity


- how to do movement arc ?
   - give precision of steps to do from 0 to 1, start to end
   - position at a given [0, 1] coef is :
      - partial rotation : (end_rotat - start rotat) * coef = p_rot
      - current center of rotation : (center of rotation) = ccr
      - p_rot.rotate(start - ccr) + speed * coef + start

# more vehicle physics :

- different drag coefficients based on ground and how the locomotion equipment interacts with it
   - applied on first part of tick when engaging locomotion or following speed


- properly following speed and applying it to relevant equipment
   - take speed vector and apply it on all ground equipment currently on ground
   - has to ignore Z axis of speed
   - project XY speed to the vehicle's X axis ? if speed is sideways the wheels shouldn't affect it
   - how to handle the speed change ?
   - use dot ?

- emulate different gears
   - different max throttles based on chosen gear
   - impossible to throttle at a given gear if speed is too low
   - cool