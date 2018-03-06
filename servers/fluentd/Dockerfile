FROM fluent/fluentd:v1.1.1-onbuild
# requires fluent.conf and plugins/ to ONBUILD COPY

RUN apk add --update --virtual .build-deps \
    sudo build-base ruby-dev \
    && sudo gem install fluent-plugin-elasticsearch --no-rdoc --no-ri --version 2.4.1 \
    && sudo gem sources --clear-all \
    && apk del .build-deps \
    && rm -rf /var/cache/apk/* /home/fluent/.gem/ruby/2.3.0/cache/*.gem
