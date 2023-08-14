#!/usr/bin/env bash
psql -U $USER -d postgres -c 'DROP DATABASE test WITH(FORCE);'
psql -U $USER -d postgres -c 'CREATE DATABASE test;'
sqlx migrate run;
