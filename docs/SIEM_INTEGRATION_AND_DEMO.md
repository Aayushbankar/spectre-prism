# SIEM Live Streaming Integration

## Overview
PRISM streams normalized OCSF events to Elasticsearch in real-time. This integration relies on Elasticsearch's `_bulk` API.

## NDJSON Format
Events are sent via HTTP POST to `/_bulk` using the `application/x-ndjson` content type.
The format consists of an action/metadata line followed by the document source, repeated for each event:
```json
{"index":{}}
{"activity_id":1,"category_uid":4,...}
{"index":{}}
{"activity_id":2,"category_uid":4,...}
```
If errors occur during insertion, the sink checks the response body for `\"errors\": true` to ensure bulk failures are detected.

## Running the Integration
1. Bring up Elasticsearch and Kibana:
```bash
docker compose up -d --wait
```
2. The `docker-compose.yml` healthcheck ensures Elasticsearch is `green` or `yellow` before marking it healthy.
3. Run PRISM and events will be pushed in bulk to the `prism-ocsf` index.

## Verification
Count ingested events:
```bash
curl -X GET "http://localhost:9200/prism-ocsf/_count"
```
In Kibana (http://localhost:5601), create an index pattern `prism-ocsf*` to view events in Discover.
