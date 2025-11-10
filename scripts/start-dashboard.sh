#!/bin/bash
# Start ccproxy cost dashboard with waitress WSGI server

cd /Users/brent/ccproxy
source venv/bin/activate

# Use waitress-serve instead of Flask dev server
waitress-serve --host=127.0.0.1 --port=8082 dashboard:app
