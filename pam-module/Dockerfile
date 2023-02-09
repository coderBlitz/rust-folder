# Change tag for versions as desired
# PAM works as-is on latest (as-of writing)
# PAM fails as-is on 16.04 ("")
FROM ubuntu:latest

COPY ./target/debug/libmyRustPam.so /lib/x86_64-linux-gnu/security/pam_myRustPam.so

RUN ["apt", "update"]
RUN ["apt", "-y", "install", "nano", "rsyslog"]
RUN ["useradd", "-M", "chris"]

RUN echo "chris:chris" | chpasswd
RUN ["sed", "-i", "/imklog/s/^/#/", "/etc/rsyslog.conf"]

CMD ["bash"]
