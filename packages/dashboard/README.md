# Control Plan API

- POST /api/deployments
  
  prepare a deployment
  - create a deployment object in database
  - create urls for functions to upload. In dev env, the returned

```bash
http POST localhost:3000/api/deployments < json_api/prepare_deploy.json
```
url is darx server's url. In prod env, the returned url is S3's url.

- POST /api/deployments/:id/bundles/:id
  - change the deployment status of a function
  - status: 'running', 'finished', 'failed'

```bash
http POST localhost:3000/api/deployments/1/bundles/1 status=success
```

- GET /api/deployments/:id/
  - get a deployment status


- GET /api/deployments

  list all deployments