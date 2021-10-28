FROM rust:alpine AS build

WORKDIR /usr/src/zkill-webhook
COPY . .

RUN apk update && apk add build-base
RUN cargo install --path .
RUN ls /usr/local/cargo/bin

FROM alpine
COPY --from=build /usr/local/cargo/bin/zkill-webhook /usr/local/bin/zkill-webhook
RUN ls /usr/local/bin

EXPOSE 3000

CMD ["zkill-webhook"]
