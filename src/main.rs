use std::{any::Any, collections::HashMap};

struct Supplier<T> {
    function: Box<dyn Fn() -> T>,
}

impl<T> Supplier<T> {
    fn new(function: Box<dyn Fn() -> T>) -> Self {
        Supplier { function }
    }
    fn get(&self) -> T {
        (self.function)()
    }
}

#[derive(Debug)]
struct Principal(String);

#[derive(Debug)]
enum ModuleResult {
    Success { principal: Option<Principal> },
    Failure,
}

trait Credentials: Any {
    fn as_any(&self) -> &dyn Any;
}

trait CredentialsValidator {
    fn new() -> Self
    where
        Self: Sized;
    fn process(&self, principal: Option<Principal>, value: &dyn Credentials) -> ModuleResult;
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
    fn process(&self, _principal: Option<Principal>, value: &dyn Credentials) -> ModuleResult {
        if let Some(credentials) = value.as_any().downcast_ref::<UserPasswordCredentials>() {
            let result = self
                .users
                .get(&credentials.user)
                .filter(|password| *password == &credentials.password);

            match result {
                Some(_) => ModuleResult::Success {
                    principal: Some(Principal(credentials.user.clone())),
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

    fn process(&self, _principal: Option<Principal>, value: &dyn Credentials) -> ModuleResult {
        if let Some(credentials) = value.as_any().downcast_ref::<u32>() {
            if *credentials == self.otp {
                ModuleResult::Success { principal: None }
            } else {
                ModuleResult::Failure
            }
        } else {
            ModuleResult::Failure
        }
    }
}

struct Module {
    credential_validator: Supplier<Box<dyn CredentialsValidator>>,
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
        credential_validator: Supplier<Box<dyn CredentialsValidator>>,
    ) {
        self.modules.push(Module {
            credential_validator,
            level_of_assurance,
        });
    }
}

struct Process<'a> {
    current: std::slice::Iter<'a, Module>,
    workflow: &'a Workflow,
    loa: u32,
    credentials_validator: Option<Box<dyn CredentialsValidator>>,
}

impl<'a> Process<'a> {
    fn new(workflow: &'a Workflow, loa: u32) -> Self {
        let mut current = workflow.modules.iter();
        let current_task = current.next().map(|m| m.credential_validator.get());
        Process {
            workflow,
            current,
            loa,
            credentials_validator: current_task,
        }
    }

    fn process(&mut self, credential: &dyn Credentials) {
        if let Some(cred) = &self.credentials_validator {
            println!("process validation workflow");
            let result = cred.process(None, credential);
            println!("process result {:?}", result);
            match result {
                ModuleResult::Success { principal: _ } => {
                    self.credentials_validator =
                        self.current.next().map(|m| m.credential_validator.get())
                }
                ModuleResult::Failure => (),
            }
        };
    }
}

impl<'a> Iterator for Process<'a> {
    type Item = &'a Module;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.next()
    }
}

// impl<T: Credentials> Module<T> {
//     pub fn process() -> ModuleResult {
//         todo!()
//     }
// }

fn main() {
    let user_password_once_cell: Supplier<Box<dyn CredentialsValidator>> =
        Supplier::new(Box::new(|| {
            Box::new(UserPasswordCredentialsValidator::new())
        }));

    let otp_validator_once_cell: Supplier<Box<dyn CredentialsValidator>> =
        Supplier::new(Box::new(|| Box::new(OtpValidator::new())));

    let mut workflow = Workflow::new();
    workflow.add_credential_validator(1, user_password_once_cell);
    workflow.add_credential_validator(2, otp_validator_once_cell);

    let result_1 = workflow.modules[0].credential_validator.get().process(
        None,
        &UserPasswordCredentials {
            user: "jean".to_string(),
            password: "pass".to_string(),
        },
    );
    println!(
        "validation du user / password : {:?}, with level {}",
        result_1, workflow.modules[0].level_of_assurance
    );
    let result_2 = workflow.modules[1]
        .credential_validator
        .get()
        .process(None, &12);
    println!(
        "validation otp : {:?} with level {}",
        result_2, workflow.modules[1].level_of_assurance
    );
    let mut p1 = Process::new(&workflow, 2);
    p1.process(&UserPasswordCredentials {
        user: "jean".to_string(),
        password: "pass".to_string(),
    });
}
