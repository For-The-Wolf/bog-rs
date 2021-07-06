# bog-rs
BogChamp: word game web-app written in rust.

Play now at [BogChamp.io](https://bogchamp.io)

Includes: 
 - `bog-rs`. A backend which generates the character grid. and 'solves' the grid efficiently using a trie data structure.
 - `BogChamp`. A frontend web app built using `actix-web` and `tera` to serve a dynamic webpage. 

![BogChamp](https://github.com/For-The-Wolf/bog-rs/blob/master/readme_images/boggers.png)

## To do
 - Fix shared state across multiple sessions
 - Multiplayer rooms
