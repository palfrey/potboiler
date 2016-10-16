FROM python:3.5.2
WORKDIR /code
ADD requirements.txt /code/requirements.txt
RUN pip install -r requirements.txt
ADD . /code
ENV FLASK_DEBUG=1
ENV FLASK_APP=app.py
CMD flask run --host=0.0.0.0
