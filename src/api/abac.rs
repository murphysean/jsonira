pub enum Decision {
    Permit,
    Deny,
    NotApplicable,
    Indeterminate,
}

pub trait PolicyDecisionPoint<E, S, R, A> {
    fn evaluate(environment: E, subject: S, resource: R, action: A) -> Decision;
}

pub struct PIPError {}

pub trait PolicyInformationPoint<E, S, R, A> {
    fn get_environment(&self) -> Result<E, PIPError>;
    fn get_subject(&self) -> Result<S, PIPError>;
    fn get_resource(&self) -> Result<R, PIPError>;
    fn get_action(&self) -> Result<A, PIPError>;
}

/*
trait PolicyEnforcementPoint{

}
*/
