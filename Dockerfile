# https://github.com/docker-library/rabbitmq/blob/master/3.8/alpine/Dockerfile
FROM rabbitmq:management-alpine

RUN apk upgrade --update && apk add --no-cache python3 python3-dev py3-pip gcc gfortran freetype-dev musl-dev libpng-dev g++ lapack-dev

RUN pip3 install -U setuptools wheel

RUN mkdir -p /home/checker
RUN mkdir -p /home/checker/example

ADD ./alarm_assert /home/checker/alarm_assert
ADD ./front_end /home/checker/front_end
ADD ./config.ini /home/checker/config.ini
ADD ./run.sh /home/checker/run.sh
ADD ./example/checker.db /home/checker/example/checker.db

ADD ./crontab_script.sh /home/checker/crontab_script.sh
ADD ./checker-crontab /etc/cron.d/checker-crontab

RUN chmod 0644 /etc/cron.d/checker-crontab &&\
    crontab /etc/cron.d/checker-crontab

RUN cd /home/checker/alarm_assert/ && pip3 install -e .
RUN cd /home/checker/front_end/ && pip3 install -e .

CMD python3 /home/checker/front_end/server/app.py

EXPOSE 5000 4369 5671 5672 15691 15692 25672