# Control Plan API

- POST /api/deployments
  
  prepare a deployment
  - create a deployment object in database
  - create urls for functions to upload. In dev env, the returned
url is darx server's url. In prod env, the returned url is S3's url.

- POST /api/deployments/:id/
  - change the deployment status of a deployment
  - status: 'running', 'finished', 'failed'

- GET /api/deployments/:id/
  - get a deployment status


- GET /api/deployments

  list all deployments