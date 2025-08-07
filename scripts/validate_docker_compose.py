
import yaml
import sys

def validate_docker_compose(file_path):
    with open(file_path, 'r') as f:
        compose_config = yaml.safe_load(f)

    total_cpus = 0.0
    total_memory_mb = 0.0

    if 'services' not in compose_config:
        print("Error: 'services' section not found in docker-compose.yml")
        sys.exit(1)

    for service_name, service_config in compose_config['services'].items():
        if 'deploy' in service_config and 'resources' in service_config['deploy'] and 'limits' in service_config['deploy']['resources']:
            limits = service_config['deploy']['resources']['limits']
            if 'cpus' in limits:
                total_cpus += float(limits['cpus'])
            if 'memory' in limits:
                memory_str = limits['memory']
                if memory_str.endswith('MB'):
                    total_memory_mb += float(memory_str[:-2])
                elif memory_str.endswith('GB'):
                    total_memory_mb += float(memory_str[:-2]) * 1024
                else:
                    print(f"Warning: Unknown memory unit for service {service_name}: {memory_str}. Skipping memory validation for this service.")

    max_cpus = 1.5
    max_memory_mb = 350.0

    print(f"Total CPUs: {total_cpus:.2f}")
    print(f"Total Memory: {total_memory_mb:.2f}MB")

    if total_cpus > max_cpus:
        print(f"Error: Total CPU limit ({total_cpus:.2f}) exceeds the maximum allowed ({max_cpus:.2f})")
        sys.exit(1)

    if total_memory_mb > max_memory_mb:
        print(f"Error: Total memory limit ({total_memory_mb:.2f}MB) exceeds the maximum allowed ({max_memory_mb:.2f}MB)")
        sys.exit(1)

    print("Docker Compose resource limits validated successfully.")

if __name__ == "__main__":
    validate_docker_compose('docker-compose.yml')
