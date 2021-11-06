FROM rust:alpine AS build

WORKDIR /usr/src/zkill-webhook
COPY ./zkill-webhook ./zkill-webhook/
COPY ./sled-ext ./sled-ext/

RUN apk update && apk add build-base
RUN cargo install --path ./zkill-webhook

FROM alpine
COPY --from=build /usr/local/cargo/bin/zkill-webhook /usr/local/bin/zkill-webhook

EXPOSE 3000

CMD ["zkill-webhook"]
