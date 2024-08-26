IMAGE_NAME := postgres:latest
CONTAINER_NAME := postgres-dev

.PHONY: help
help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@echo "  setup-dev         Pulls Postgres image and runs the container"
	@echo "  pull-image        Pulls the latest Postgres image"
	@echo "  run-container     Runs Postgres in a container"
	@echo "  clean             Stops and removes the Postgres container"
	@echo "  logs              Shows the logs of the container"
	@echo "  restart           Cleans and restarts the Postgres container"

.PHONY: setup-dev
setup-dev: pull-image run-container

.PHONY: pull-image
pull-image:
	docker pull $(IMAGE_NAME)

.PHONY: run-container
run-container:
	docker run --name $(CONTAINER_NAME) \
		-e POSTGRES_PASSWORD=password \
		-p 5432:5432 \
		-d $(IMAGE_NAME)

	@echo "Postgres URL: postgres://postgres:password@0.0.0.0:5432/postgres"

.PHONY: clean
clean:
	docker stop $(CONTAINER_NAME) || true
	docker rm $(CONTAINER_NAME) || true

.PHONY: logs
logs:
	docker logs $(CONTAINER_NAME)

.PHONY: restart
restart: clean run-container
