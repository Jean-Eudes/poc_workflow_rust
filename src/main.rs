use std::{any::Any, collections::HashMap};

#[derive(Debug)]
enum ModuleResult {
    Success { user_name: Option<String> },
    Failure,
}

trait Credentials: Any {
    fn as_any(&self) -> &dyn Any;
}

trait CredentialsValidator {
    fn new() -> Self
    where
        Self: Sized;
    fn process(&self, value: &dyn Credentials) -> ModuleResult;
}

struct UserPasswordCredentialsValidator {
    users: HashMap<String, String>,
}

struct UserPasswordCredentials {
    user: String,
    password: String,
}

impl Credentials for UserPasswordCredentials {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl Credentials for u32 {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CredentialsValidator for UserPasswordCredentialsValidator {
    fn process(&self, value: &dyn Credentials) -> ModuleResult {
        if let Some(credentials) = value.as_any().downcast_ref::<UserPasswordCredentials>() {
            let result = self
                .users
                .get(&credentials.user)
                .filter(|password| *password == &credentials.password);

            match result {
                Some(_) => ModuleResult::Success {
                    user_name: Some(credentials.user.clone()),
                },
                None => ModuleResult::Failure,
            }
        } else {
            return ModuleResult::Failure;
        }
    }

    fn new() -> Self {
        let mut service = UserPasswordCredentialsValidator::default();
        service.users.insert("jean".to_string(), "pass".to_string());
        service
    }
}

impl Default for UserPasswordCredentialsValidator {
    fn default() -> Self {
        UserPasswordCredentialsValidator {
            users: Default::default(),
        }
    }
}

struct OtpValidator {
    otp: u32,
}

impl CredentialsValidator for OtpValidator {
    fn new() -> Self
    where
        Self: Sized,
    {
        OtpValidator { otp: 12 }
    }

    fn process(&self, value: &dyn Credentials) -> ModuleResult {
        if let Some(credentials) = value.as_any().downcast_ref::<u32>() {
            if *credentials == self.otp {
                ModuleResult::Success { user_name: None }
            } else {
                ModuleResult::Failure
            }
        } else {
            ModuleResult::Failure
        }
    }
}

struct Module {
    credential_validator: Box<dyn CredentialsValidator>,
    level_of_assurance: u32,
}

struct Workflow {
    modules: Vec<Module>,
}

impl Workflow {
    fn new() -> Self {
        Workflow { modules: vec![] }
    }

    fn add_credential_validator(
        &mut self,
        level_of_assurance: u32,
        credential_validator: Box<dyn CredentialsValidator>,
    ) {
        self.modules.push(Module {
            credential_validator,
            level_of_assurance,
        });
    }
}

// impl<T: Credentials> Module<T> {
//     pub fn process() -> ModuleResult {
//         todo!()
//     }
// }

fn main() {
    let user_password_service: Box<dyn CredentialsValidator> =
        Box::new(UserPasswordCredentialsValidator::new());

    let otp_validator = OtpValidator::new();

    let mut workflow = Workflow::new();
    workflow.add_credential_validator(1, user_password_service);
    workflow.add_credential_validator(2, Box::new(otp_validator));

    let result_1 = workflow.modules[0]
        .credential_validator
        .process(&UserPasswordCredentials {
            user: "jean".to_string(),
            password: "pass".to_string(),
        });
    println!(
        "validation du user / password : {:?}, with level {}",
        result_1, workflow.modules[0].level_of_assurance
    );
    let result_2 = workflow.modules[1].credential_validator.process(&12);
    println!(
        "validation otp : {:?} with level {}",
        result_2, workflow.modules[1].level_of_assurance
    );
}
