# testcontainer-rust

## Windows

If `cargo test` don't work you need to:

-   set `DOCKER_HOST="tcp://localhost:2375"` env variable (you can user a .env file in root directory)
-   enable Docker Desktop "Expose daemon on tcp://localhost:2375 without TLS" option inside "General" Settings
