# Download binary for backend and run it

FROM debian:bookworm

EXPOSE 3001

RUN apt-get -y update; apt-get -y install curl wget

RUN curl -s https://api.github.com/repos/gitbounties/backend/releases/latest \
    | grep "browser_download_url" \
    | cut -d : -f 2,3 \
    | tr -d \" \
    | wget -qi -

RUN chmod 755 ./gitbounties_backend

CMD ["./gitbounties_backend"]
