use colored::Colorize;
use std::env;
use std::process::Command;

fn main() {
    let arguments: Vec<String> = env::args().collect();
    match arguments.get(1) {
        Some(policy_path) => {
            if policy_path == "help" {
                print_error();
                return;
            }
            match arguments.get(2) {
                Some(project_id) => match arguments.get(3) {
                    Some(gateway_name) => match arguments.get(4) {
                        Some(region) => {
                            println!(
                                "{} {} {} {} {} {} {} {}\n{}",
                                "Preparing to deploy the given ESPv2 Policy from path:".red(),
                                policy_path.green(),
                                "in project".red(),
                                project_id.green(),
                                "with cloud run container name".red(),
                                gateway_name.green(),
                                "to region".red(),
                                region.green(),
                                "Please wait...".blue()
                            );
                            deploy_gateway(policy_path, project_id, gateway_name, region);
                        }
                        None => print_error(),
                    },
                    None => print_error(),
                },
                None => print_error(),
            }
        }
        None => print_error(),
    }
}

fn deploy_gateway(policy_path: &str, project_id: &str, gateway_name: &str, region: &str) {
    let endpoints_deploy = Command::new("gcloud")
        .arg("endpoints")
        .arg("services")
        .arg("deploy")
        .arg(policy_path)
        .arg("--project")
        .arg(project_id)
        .output()
        .expect("Failed to execute process!");
    let config_label = get_config_label(std::str::from_utf8(&endpoints_deploy.stderr).unwrap());
    let service_label = get_service_label(std::str::from_utf8(&endpoints_deploy.stderr).unwrap());
    println!("{}", "Please wait...".blue());
    let build_image = Command::new("./gcloud_build_image")
        .arg("-s")
        .arg(service_label)
        .arg("-c")
        .arg(config_label)
        .arg("-p")
        .arg(project_id)
        .output()
        .expect("Cannot find and execute ./gcloud_build_image.sh (Make sure that build_script is in the same directory as the executable)");
    let build_image = get_build_image(
        std::str::from_utf8(&build_image.stderr).unwrap(),
        config_label,
        project_id,
    );
    println!("{}", "Deploying build image...".blue());
    let deploy_image = Command::new("gcloud")
        .arg("run")
        .arg("deploy")
        .arg(gateway_name)
        .arg(format!("--image={}", format!("{}", build_image).as_str()))
        .arg("--allow-unauthenticated")
        .arg("--platform")
        .arg("managed")
        .arg(format!("--region={}", region))
        .arg(format!("--project={}", project_id))
        .output()
        .expect("Failed to execute process!");
    println!(
        "{}\n{}",
        "Deploy finished with the following message:".green(),
        std::str::from_utf8(&deploy_image.stderr).unwrap()
    );
}

fn print_error() {
    println!(
        "{}",
        "Usage: ./itb-deploy-gateway <path_to_espv2_yaml_definition> <project_id> <gateway_name> <region (e.g. us-central1)>"
            .bright_red()
    )
}

fn get_config_label(stdout: &str) -> &str {
    let start_index = stdout
        .find("Service Configuration [")
        .expect("No config label found with &str `Service Configuration [`");
    let end_index = stdout
        .find("] uploaded for service [")
        .expect("No config label found with &str `] uploaded for service [`");
    let config_label = &stdout[start_index + 23..end_index];
    println!("{} {}", "Found config label:".red(), config_label.green());
    return config_label;
}

fn get_service_label(stdout: &str) -> &str {
    let start_index = stdout
        .find("] uploaded for service [")
        .expect("No service label found with &str `] uploaded for service [`");
    let end_index = stdout
        .find(".run.app]")
        .expect("No service label found with &str `.run.app]`");
    let service_label = &stdout[start_index + 24..end_index + 8];
    println!("{} {}", "Found service label:".red(), service_label.green());
    return service_label;
}

fn get_build_image<'a>(stderr: &'a str, config_label: &str, project_id: &str) -> &'a str {
    let start_index = stderr
        .find(format!("gcr.io/{}", project_id).as_str())
        .expect("Could not locate build_image");
    let end_index = stderr
        .find(format!("{}", config_label).as_str())
        .expect("Could not locate build_image");

    let build_image = &stderr[start_index..end_index + config_label.len()];
    println!("{} {}", "Found build image:".red(), build_image.green());
    return build_image;
}
