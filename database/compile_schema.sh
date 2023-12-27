#!/bin/bash
dbml2sql ./schema.dbml -o ./schema.sql

rm ./dbml-error.log
