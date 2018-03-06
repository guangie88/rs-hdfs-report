#!/usr/bin/env sh
USER=admin
PW=admin
RETRY_COUNT=50

is_valid_health_status() {
    status=`curl -s -u $USER:$PW -H "Content-Type: application/json" \
        -X GET http://grafana:3000/api/health \
        --retry $RETRY_COUNT | jq -r '.database'`
    echo "$status"
}

# check for health status before running the data source + dashboard imports
is_valid=""
until [ "$is_valid" = "ok" ]; do
    sleep 1
    is_valid=$(is_valid_health_status)
done

curl -s -i -u $USER:$PW -H "Content-Type: application/json" \
    -X POST http://grafana:3000/api/datasources \
    -d '{"name":"elasticsearch","type":"elasticsearch","url":"http://elasticsearch:9200", "access":"proxy", "database":"logstash-*","jsonData":{"timeField":"datetime"}}' \
    --retry $RETRY_COUNT > /dev/null

curl -s -i -u $USER:$PW -H "Content-Type: application/json" \
    -X POST http://grafana:3000/api/dashboards/import \
    -d '{"overwrite":true,"inputs":[{"name":"DS_ELASTICSEARCH","type":"datasource","pluginId":"elasticsearch","value":"elasticsearch"}],"dashboard":'"`cat /diagnostics.json | jq . -c`"'}' \
    --retry $RETRY_COUNT > /dev/null
