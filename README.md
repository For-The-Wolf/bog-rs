# bog-rs
BogChamp: a multi-player word game web-app written in rust.

Play now at [BogChamp.io](https://bogchamp.io)

Includes: 
 - `bog-rs`. A backend which generates the character grid. and 'solves' the grid efficiently using a trie.
 - `BogChamp`. A frontend web app built using `actix-web` and `tera` to serve a dynamic webpage. 

![BogChamp](https://github.com/For-The-Wolf/bog-rs/blob/master/readme_images/boggers.png)

## To do
 - [x] Fix shared state across multiple sessions.
 - [x] Multiplayer beta (rooms, real-time scoring, synchronised timing, etc.).
 - [x] Multiplayer stable release (inc. pretty UI).
 - [x] Fix bug with CSS in FireFox 
 - [ ] Update server to store state in concurrent hashmap (`DashMap< >`) rather than `Mutex<HashMap< >>`.
 - [ ] Use `actix_actors::ws` for Websockets
