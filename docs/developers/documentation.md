# gitehr.org

Documentation and specification site for GitEHR: https://gitehr.org

## Development Getting Started

Dockerized development, running Material for MkDocs. Get started in 3 steps:

Clone the repo:

```bash
git clone https://github.com/gitehr/gitehr.org.git
```

Enter the directory:

```bash
cd gitehr.org
```

Run the following Docker command:

```bash
docker run --rm -it -p 8000:8000 -v ${PWD}:/docs squidfunk/mkdocs-material
```

See the local development site at `localhost:8000`.
