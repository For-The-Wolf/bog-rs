# bog-rs
BogChamp: word game web-app written in rust.

Includes: 
 - `bog-rs`. A backend which generates the character grid. and 'solves' the grid by finding all the valid words.
 - `BogChamp`. A frontend web app built using `actix-web` and `tera` to serve a dynamic webpage. 

Many advantages over the traditional game, including a sensible letter distribution, very efficient solver using a trie, and upcoming support for the Oxford English Dictionary API to ensure all solutions are valid.

I hope to host this somewhere soon.

![BogChamp](https://github.com/For-The-Wolf/bog-rs/blob/master/readme_images/bog-rs.png)
