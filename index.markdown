---
# Feel free to add content and custom Front Matter to this file.
# To modify the layout, see https://jekyllrb.com/docs/themes/#overriding-theme-defaults

layout: home
---

**Fitness-tracker** is my first attempt with the ***Rust*** programming language.  
It is a simple offline web application that visualizes the frequency of workouts  
based on user input.

# Features
* It is built in **Rust** and plain **HTML**
* It uses **Actix Web** for the backend
* It uses **Sqlite** as an offline local means of storing user information.

# Instructions

to compile the project you need to install the rust programming language and cargo.  

After installing the above dependencies, you can simple building the project by:
```sh
cargo build
```

to run the project simply:
```sh
cargo run
```

You will be able to access the webpage at [http://localhost:8080](http://localhost:8080)

# Internals

1. At program startup a connection to the sqlite database is established.
2. When a client requests `/` for the application a static website is served.
3. The website then requests workout information from the `/workouts` endpoint and displays it on the heatmap.
4. The user can input more workouts or update the already existing ones by using the `/add` endpoint.



