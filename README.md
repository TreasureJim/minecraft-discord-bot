# Environment Variables

`DISCORD_TOKEN` - discord token to authenticate to discord
`CONTAINER_NAME` - the name of the container that the bot should monitor

# Development

The code will try to read a .env file in the current directory during development to load environment variables. 
Since the program needs to be pointed to a running container an example `docker-compose.dev.yaml` file has been provided. 
