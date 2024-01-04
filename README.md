# teams-league-cloudrun-rust

This project shows a complete Rust program with a real world use case, deployed in Cloud Run. 
The program read a raw file from GCS and write the result to BigQuery.

I share the link to the article with the same use case but with Python :

https://medium.com/@mazlum.tosun/cloud-run-service-with-a-python-module-fastapi-and-uvicorn-24c94090a008

## Build the Cloud Build job from the local machine 

```bash
gcloud builds submit \
    --project=$PROJECT_ID \
    --region=$LOCATION \
    --config deploy-cloud-run-rust-service.yaml \
    --substitutions _REPO_NAME="$REPO_NAME",_SERVICE_NAME="$SERVICE_NAME",_IMAGE_TAG="$IMAGE_TAG",_OUTPUT_DATASET="$OUTPUT_DATASET",_OUTPUT_TABLE="$OUTPUT_TABLE",_INPUT_BUCKET="$INPUT_BUCKET",_INPUT_OBJECT="$INPUT_OBJECT" \
    --verbosity="debug" .
```