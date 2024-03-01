# Change tag for versions as desired
FROM ubuntu:14.04

#COPY ./target/debug/libpam_exec.so /lib/x86_64-linux-gnu/security/pam_exec.so
COPY ./libpam_exec.so /lib/x86_64-linux-gnu/security/pam_exec.so

RUN ["apt", "update"]
RUN ["apt", "-y", "install", "nano", "rsyslog", "libpam0g"]
RUN ["useradd", "-M", "chris"]

RUN echo "chris:chris" | chpasswd
RUN ["sed", "-i", "/imklog/s/^/#/", "/etc/rsyslog.conf"]

CMD ["bash"]
