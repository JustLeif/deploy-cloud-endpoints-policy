use colored::Colorize;
use std::env;
use std::process::Command;
struct Arguments<'a> {
    path_to_open_api_specification: Option<&'a str>,
    path_to_image_build_script: Option<&'a str>,
    project_id: Option<&'a str>,
    cloud_run_service_name: Option<&'a str>,
    region: Option<&'a str>,
}
impl<'a> Arguments<'a> {
    fn new() -> Arguments<'a> {
        Arguments {
            path_to_open_api_specification: None,
            path_to_image_build_script: None,
            project_id: None,
            cloud_run_service_name: None,
            region: None,
        }
    }
    fn has_all_arguments(&self) -> bool {
        return self.path_to_open_api_specification.is_some()
            && self.path_to_image_build_script.is_some()
            && self.project_id.is_some()
            && self.cloud_run_service_name.is_some()
            && self.region.is_some();
    }
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    if &arguments[1] == "--help" {
        print_help();
    } else {
        match parse_arguments(&arguments) {
            Ok(args) => {
                deploy_policy(args);
            }
            Err(message) => {
                println!("{}", message);
                print_help();
            }
        }
    }
}

fn deploy_policy(arguments: Arguments) {
    println!(
        "{} {} {} {} {} {} {} {}\n{}",
        "Preparing to deploy the given ESPv2 Policy from path:".red(),
        &arguments.path_to_open_api_specification.unwrap().green(),
        "in project".red(),
        &arguments.project_id.unwrap().green(),
        "with cloud run service name".red(),
        &arguments.cloud_run_service_name.unwrap().green(),
        "to region".red(),
        &arguments.region.unwrap().green(),
        "Please wait...".blue()
    );

    let endpoints_deploy = Command::new("gcloud")
        .arg("endpoints")
        .arg("services")
        .arg("deploy")
        .arg(&arguments.path_to_open_api_specification.unwrap())
        .arg("--project")
        .arg(&arguments.project_id.unwrap())
        .output()
        .expect("Failed to execute process!");
    let config_label = get_config_label(std::str::from_utf8(&endpoints_deploy.stderr).unwrap());
    let service_label = get_service_label(std::str::from_utf8(&endpoints_deploy.stderr).unwrap());
    println!("{}", "Please wait...".blue());
    let build_image = Command::new(&arguments.path_to_image_build_script.unwrap())
        .arg("-s")
        .arg(service_label)
        .arg("-c")
        .arg(config_label)
        .arg("-p")
        .arg(&arguments.project_id.unwrap())
        .output()
        .expect("Cannot find and execute ./gcloud_build_image.sh (Make sure that build_script is in the same directory as the executable)");
    let build_image = get_build_image(
        std::str::from_utf8(&build_image.stderr).unwrap(),
        config_label,
        &arguments.project_id.unwrap(),
    );
    println!("{}", "Please wait...".blue());
    let deploy_image = Command::new("gcloud")
        .arg("run")
        .arg("deploy")
        .arg(&arguments.cloud_run_service_name.unwrap())
        .arg(format!("--image={}", format!("{}", build_image).as_str()))
        .arg("--allow-unauthenticated")
        .arg("--platform")
        .arg("managed")
        .arg(format!("--region={}", &arguments.region.unwrap()))
        .arg(format!("--project={}", &arguments.project_id.unwrap()))
        .output()
        .expect("Failed to execute process!");
    println!(
        "{}\n{}",
        "Deploy finished with the following message:".green(),
        std::str::from_utf8(&deploy_image.stderr).unwrap()
    );
}

fn parse_arguments<'a>(arguments: &'a Vec<String>) -> Result<Arguments<'a>, String> {
    let mut args = Arguments::new();
    for (index, arg) in arguments.into_iter().enumerate() {
        match arg.as_str() {
            "--yaml-path" => {
                args.path_to_open_api_specification = Some(arguments[index + 1].as_str())
            }
            "--build-script-path" => {
                args.path_to_image_build_script = Some(arguments[index + 1].as_str())
            }
            "--project-id" => args.project_id = Some(arguments[index + 1].as_str()),
            "--cloud-run-service-name" => {
                args.cloud_run_service_name = Some(arguments[index + 1].as_str())
            }
            "--region" => args.region = Some(arguments[index + 1].as_str()),
            _ => {}
        }
    }
    if !args.has_all_arguments() {
        return Err("Not all arguments were provided. Please provide all arguments".to_string());
    }
    return Ok(args);
}

fn print_help() {
    println!(
        "{}\n\t{} {}\n\t{} {}\n\t{} {}\n\t{} {}\n\t{} {}",
        "Usage: ./<bin_name>".bright_red(),
        "--yaml-path".green(),
        "<path_to_openapi_spec>".blue(),
        "--build-script-path".green(),
        "<path_to_gcloud_build_image>".blue(),
        "--project-id".green(),
        "<project_id>".blue(),
        "--cloud-run-service-name".green(),
        "<gateway_name>".blue(),
        "--region".green(),
        "<region_id>".blue()
    );
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
