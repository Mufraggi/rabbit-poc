docker cmd mongo

docker run -d -p 27017:27017 --name test-mongo mongo:latest

docker cmd rabbit

docker run -p 15672:15672 -p 5672:5672 -e RABBITMQ_DEFAULT_USER=rmq -e RABBITMQ_DEFAULT_PASS=rmq  rabbitmq:3.8.4-management