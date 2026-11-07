#!/usr/bin/env bash
# Build the dikumud image for arm64, push it to ECR as :latest and :<git sha>,
# then restart the game on the server so it pulls the new image.
#
# Players still online lose anything since they last saved: have an immortal
# `shutdown` in the game first if anyone is on.
#
# Usage: AWS_PROFILE=diku-deploy deploy/aws/deploy.sh
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
region=${AWS_REGION:-us-east-2}

repo=$(tofu -chdir="$here" output -raw ecr_repository_url)
instance=$(tofu -chdir="$here" output -raw instance_id)
registry=${repo%%/*}
sha=$(git -C "$root" rev-parse --short HEAD)
[ -z "$(git -C "$root" status --porcelain)" ] || sha="$sha-dirty"

aws ecr get-login-password --region "$region" |
  docker login --username AWS --password-stdin "$registry"

docker buildx build --platform linux/arm64 \
  -t "$repo:$sha" -t "$repo:latest" --push "$root"

echo "Restarting dikumud on $instance ..."
cmd=$(aws ssm send-command --region "$region" \
  --instance-ids "$instance" \
  --document-name AWS-RunShellScript \
  --comment "deploy dikumud $sha" \
  --parameters 'commands=["systemctl restart dikumud","sleep 5","systemctl is-active dikumud","docker inspect -f {{.Config.Image}}@{{.Image}} dikumud"]' \
  --query Command.CommandId --output text)

aws ssm wait command-executed --region "$region" --command-id "$cmd" --instance-id "$instance" 2>/dev/null || true
read -r status < <(aws ssm get-command-invocation --region "$region" --command-id "$cmd" \
  --instance-id "$instance" --query Status --output text)
aws ssm get-command-invocation --region "$region" --command-id "$cmd" --instance-id "$instance" \
  --query '[StandardOutputContent, StandardErrorContent]' --output text

if [ "$status" != Success ]; then
  echo "Restart failed ($status). On a brand-new instance, cloud-init may still be" >&2
  echo "installing Docker: the service picks the image up by itself once it's done." >&2
  exit 1
fi
echo "Deployed $sha: $(tofu -chdir="$here" output -raw url)"
