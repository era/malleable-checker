# https://github.com/docker-library/rabbitmq/blob/master/3.8/alpine/Dockerfile
FROM rabbitmq:alpine

RUN apk upgrade --update && apk add --no-cache python3 python3-dev py3-pip gcc gfortran freetype-dev musl-dev libpng-dev g++ lapack-dev

RUN pip3 install -U setuptools wheel

RUN mkdir -p /home/checker

ADD ./alarm_assert /home/checker/alarm_assert
ADD ./front_end /home/checker/front_end

RUN cd /home/checker/alarm_assert/ && python3 setup.py install
RUN cd /home/checker/front_end/ && python3 setup.py install