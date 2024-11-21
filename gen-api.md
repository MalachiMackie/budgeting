# Generating api

```
cd ./budgeting-backend
cargo run -- gen-swagger ../api-doc.json
```

Now you need to go into api-doc.json and modify it so any `"nullable": true` are set to false. see https://github.com/openapistack/openapicmd/issues/63

```
cd ../budgeting-ui
npm run gen-api-client
```
