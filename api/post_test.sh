#!/bin/sh

curl --location --request POST 'localhost:3030/api/post_job' \
    --header 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJ1dWlkIjoiMDE4Y2MwMTZjNGVmNzFlMDlmMTYyYzk3OGIzZTVhYjMiLCJyb2xlIjoiQ29tcGFueSIsImV4cCI6MTcwNDYzNTUxOH0.jKUMfX0L9VdUpgU6EWd7cXThHhrwV00qOJAHikm4p01FnaivCu_2RyyWQB9jsRJAitiwJ1mVRl3ccq-aTQN0NQ'\
    --header 'Content-Type: application/json' \
    --header 'Content-Type: text/plain' \
    --data-raw '{
        "job_location": "Somewhere",
        "contract_type": "Zlecenie",
        "mode": "Mobile",
        "hours": "Week",
        "tags": ["Week", "Mobile", "Zlecenie", "random"],
        "description": "A test job listing for testing"
    }'