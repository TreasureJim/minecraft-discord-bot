d-build:
	docker build --progress plain -t treasurejim/minecraft-discord-bot:latest .

d-push:
	docker push treasurejim/minecraft-discord-bot:latest
