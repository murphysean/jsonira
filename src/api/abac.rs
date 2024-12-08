trait PolicyDecisionPoint<T>{
    fn evaluate(environment: T, subject: T, resource: T, action: T) -> Decision;
}

trait PolicyInformationPoint(

)

trait PolicyEnforcementPoint{

}