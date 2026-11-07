# Deploying dikumud to AWS

This runs <https://diku.earth/> from AWS account 767398022292 in us-east-2.
The infrastructure is [OpenTofu](https://opentofu.org/) 1.12.5 (see
`.opentofu-version`).

```
Internet ──► Elastic IP ──► t4g.micro (Amazon Linux 2023, arm64)
               ▲              ├─ caddy     :80/:443, Let's Encrypt TLS, WebSockets
   diku.earth  │              └─ dikumud   :4000 on a private Docker network
   www ────────┘              /srv/dikumud = separate encrypted EBS volume
                                (player files, board, Caddy certificates),
                                snapshotted daily, kept 14 days
   container output ──► CloudWatch Logs group `dikumud`, kept 30 days
```

There's no SSH. Get a shell through SSM Session Manager.

| File | What it is |
|---|---|
| `claude-deploy-role.cfn.yaml` | CloudFormation stack `dikumud-deploy-access`: the OpenTofu state bucket and the scoped `claude-dikumud-deploy` role |
| `*.tf`, `cloud-init.yaml.tftpl` | Network, instance, data volume, snapshots, ECR repository, log group, DNS |
| `deploy.sh` | Builds the arm64 image, pushes it to ECR and restarts the game |

## Access

Profiles in `~/.aws/config`:

- `diku-sso` is your IAM Identity Center session (AdministratorAccess). Use it
  only to deploy or change the CloudFormation stack.
- `diku-deploy` assumes `claude-dikumud-deploy` from `diku-sso`. Use it for
  everything else. It can only touch this project's resources in us-east-2
  and the `diku.earth`/`www.diku.earth` records. The template lists the
  limits.

```bash
aws sso login --sso-session diku
export AWS_PROFILE=diku-deploy
```

The stack was created once with:

```bash
aws cloudformation deploy --profile diku-sso --region us-east-2 \
  --stack-name dikumud-deploy-access \
  --template-file deploy/aws/claude-deploy-role.cfn.yaml \
  --capabilities CAPABILITY_NAMED_IAM
```

## Infrastructure changes

```bash
cd deploy/aws
tofu init
tofu plan -out=dikumud.tfplan
tofu apply dikumud.tfplan
```

Changing `cloud-init.yaml.tftpl` replaces the instance. The data volume is
detached and attached to the new instance, so players, the board and the
certificates survive. The game is down for a few minutes. The data volume
has `prevent_destroy`, so OpenTofu refuses any plan that would delete it.

## New game code

From the repository root, after committing:

```bash
deploy/aws/deploy.sh
```

This kicks everyone online off. Players lose anything since they last saved.
If people are playing, have an immortal type `shutdown` in the game first:
that saves them, and systemd starts the game again. Every image is also
tagged with its git commit in ECR, for rollback.

## Operating the server

```bash
aws ssm start-session --target "$(tofu -chdir=deploy/aws output -raw instance_id)"
```

On the server:

- `systemctl status dikumud caddy` shows whether the containers are up.
  `journalctl -u dikumud` shows only start, stop and Docker errors: the
  containers' own output isn't kept on the server.
- `systemctl restart dikumud` restarts the game, pulling `:latest` first.
- Game data is in `/srv/dikumud/data`, which is mounted in the container
  at `/var/lib/dikumud`.

## Logs

Docker sends each container's output to the CloudWatch Logs group `dikumud`,
one stream per container: `dikumud` (the game log) and `caddy` (the proxy).
Nothing is written to disk on the server, and `docker logs` has nothing to
show. From your machine:

```bash
aws logs tail dikumud --log-stream-names dikumud --follow
aws logs tail dikumud --log-stream-names caddy --since 1h
```

Logs are kept for 30 days (`log_retention_days`). If CloudWatch can't be
reached, Docker holds up to 4 MB per container in memory and then drops
lines, so a logging outage never stalls the game. The host's own journal
(cloud-init, updates, dockerd) stays on the server, capped at 200 MB.

## OS updates

OS updates install every Sunday at 10:00 UTC (plus up to 30 minutes), and the
server reboots only when an update needs it. The instance keeps the AMI it was
created with. A newer AMI arrives only when the instance is replaced.

## Restoring from a snapshot

Snapshots are tagged `Snapshot=dikumud-daily`. To roll back the data, create a
volume from a snapshot in us-east-2a. Then stop the game and swap the volumes:
`tofu import` the new volume over `aws_ebs_volume.data` after removing the old
one from state with `tofu state rm`. Or mount it beside the old volume and
copy the files you need.
