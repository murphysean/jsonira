use jiff::civil::Date;
use jiff::Span;
use jiff::Timestamp;
use json_patch::{Patch, PatchOperation, ReplaceOperation};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json};

use crate::api::abac::Decision;

use super::subject::AuthContext;
use super::subject::Subject;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "reason")]
pub enum BlockedReason {
    WaitingOnOther,
    NeedsInformation,
    NeedsPlanning,
    NeedsMaterials,
    NeedsTools,
    InsufficentBudget,
    RequiresAuthorization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "resolution")]
pub enum Resolution {
    Done,
    NothingToDo,
    WontDo,
    Cancelled,
    Duplicate,
    ClosedForAge,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "state")]
pub enum TaskState {
    Blocked(BlockedReason),
    #[default]
    Todo,
    InProgress,
    InReview,
    Complete(Resolution),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskPriority {
    Neutral,
    Trivial,
    Minor,
    Major,
    Critical,
    Blocker,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Task {
    ///Assigned by the storage layer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    //Assumed to be 'text/plain' if not otherwise specified
    pub description_content_type: Option<String>,

    pub reporter: Option<Subject>,
    pub watchers: Option<Vec<Subject>>,
    //At creation a group can be assigned
    //It can be one of the creating users groups
    //Or it can be 'public' which offers this task to everyone (Everyone is a member of public)
    //If it is none, then only the reporter, assignee, and watchers will have access to this task
    pub circle: Option<String>,

    pub assignee: Option<Subject>,
    pub priority: Option<TaskPriority>,
    pub estimate: Option<Span>,
    pub points: Option<i64>,
    pub state: Option<TaskState>,

    pub tags: Option<Vec<String>>,

    pub created: Option<Timestamp>,
    pub updated: Option<Timestamp>,
    pub due: Option<Date>,

    pub comments: Option<Vec<Comment>>,
    pub reactions: Option<Vec<Reaction>>,
    pub reviews: Option<Vec<Review>>,

    pub history: Option<Vec<Action>>,
}

impl Task {
    pub fn generate_action(&self, subject: Subject) -> Action {
        let mut patch: Patch = Patch::default();

        if self.title.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/title".into()).unwrap();
            op.value = json!(self.title);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.description.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/description".into()).unwrap();
            op.value = json!(self.description);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.description_content_type.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/description_content_type".into()).unwrap();
            op.value = json!(self.description_content_type);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.reporter.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/reporter".into()).unwrap();
            op.value = json!(self.reporter);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.watchers.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/watchers".into()).unwrap();
            op.value = json!(self.watchers);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.circle.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/circle".into()).unwrap();
            op.value = json!(self.circle);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.priority.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/priority".into()).unwrap();
            op.value = json!(self.priority);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.estimate.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/estimate".into()).unwrap();
            op.value = json!(self.estimate);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.points.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/points".into()).unwrap();
            op.value = json!(self.points);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.state.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/state".into()).unwrap();
            op.value = json!(self.state);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.tags.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/tags".into()).unwrap();
            op.value = json!(self.tags);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.due.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/due".into()).unwrap();
            op.value = json!(self.due);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.comments.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/comments".into()).unwrap();
            op.value = json!(self.comments);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.reactions.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/reactions".into()).unwrap();
            op.value = json!(self.reactions);
            patch.0.push(PatchOperation::Replace(op));
        }
        if self.reviews.is_some() {
            let mut op: ReplaceOperation = ReplaceOperation::default();
            op.path = from_value("/reviews".into()).unwrap();
            op.value = json!(self.reviews);
            patch.0.push(PatchOperation::Replace(op));
        }

        Action {
            subject,
            patched: Timestamp::now(),
            patch: patch,
        }
    }

    /// Steps through the proposed patch operations and determines if it is allowed given the subject, resource, and action
    ///
    /// Lets start with some basics
    /// All users can add comments, reviews, reactions
    /// Only users in the circle (or watchers) can change (replace) task properties
    /// State changes can only be made by the assignee, and some depend on
    pub fn policy(&self, auth_context: &AuthContext, patch: &Patch) -> (Decision, &str) {
        let user_is_member = self.user_is_member(auth_context);
        let user_is_assignee = self.user_is_assignee(auth_context);
        let user_is_reporter = self.user_is_reporter(auth_context);
        
        for p in &patch.0 {
            match p {
                PatchOperation::Add(ao) => {
                    //Only allow add to the arrays via /-
                    let allowed_add_paths = vec![
                        "/comments/-",
                        "/reactions/-",
                        "/reviews/-",
                        "/tags/-",
                        "/watchers/-",
                        "/todos/-",
                    ];
                    if !allowed_add_paths.contains(&ao.path.as_str()) {
                        return (
                            Decision::Deny,
                            "Can only add to one of the tasks collections",
                        );
                    }
                    //Non members can only add comments and reactions

                }
                PatchOperation::Replace(ro) => {
                    //Must be a watcher, assignee, or member of the circle
                    if !user_is_member {
                        return (Decision::Deny, "User must be a member of the task");
                    }
                    if ro
                        .path
                        .as_str()
                        .to_owned()
                        .split("/")
                        .count()
                        > 2
                    {
                        return (Decision::Deny, "Can only replace top level items now");
                    }
                    if ro.path.as_str() == "/state" {
                        //Only the assignee or reporter can change state
                        if !user_is_assignee && !user_is_reporter {
                            return (Decision::Deny, "Must be assignee or reporter to change state");
                        }
                    }
                    let must_be_reporter_paths = vec![
                        "/title",
                        "/description",
                        "/estimate",
                        "/points",
                        "/due",
                    ];
                    if must_be_reporter_paths.contains(&ro.path.as_str()){
                        if !user_is_reporter{
                            return (Decision::Deny, "Must be reporter to change core attributes");
                        }
                    }
                }
                PatchOperation::Remove(_) => return (Decision::Deny, "Unsupported"),
                PatchOperation::Copy(_) => return (Decision::Deny, "Unsupported"),
                PatchOperation::Move(_) => return (Decision::Deny, "Unsupported"),
                PatchOperation::Test(_) => return (Decision::Deny, "Unsupported"),
            }
        }
        (Decision::Permit, "Permitted")
    }

    fn user_is_member(&self, auth_context: &AuthContext) -> bool {
        //Is the user the reporter?
        if self.user_is_reporter(auth_context){
            return true;
        }

        //Is the user assigned to the task?
        if self.user_is_assignee(auth_context){
            return true;
        }

        //Is the user a watcher?
        if self
            .watchers
            .as_ref()
            .map(|v| v.contains(&auth_context.subject))
            .unwrap_or(false)
        {
            return true;
        }

        //Is the user a member of the tasks circle?
        if auth_context.subject.in_circle(&self.circle) {
            return true;
        }
        false
    }

    fn user_is_reporter(&self, auth_context: &AuthContext) -> bool {
        if self
            .reporter
            .as_ref()
            .map(|r| *r == auth_context.subject)
            .unwrap_or(false)
        {
            return true;
        }
        false
    }
    fn user_is_assignee(&self, auth_context: &AuthContext) -> bool {
        if self
            .assignee
            .as_ref()
            .map(|a| *a == auth_context.subject)
            .unwrap_or(false)
        {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub subject: Subject,
    pub comment: String,
    //Assumed to be 'text/plain' if not specified here
    pub content_type: Option<String>,
    pub created: Timestamp,
    pub updated: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub subject: Subject,
    //You can react to anything else in the document using a json patch path here
    pub path: Option<String>,
    pub reaction: String,
    pub created: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum ReviewState {
    Reviewing,
    Approved,
    RequestChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub subject: Subject,
    pub review: ReviewState,
    pub rating: Option<Rating>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "rating")]
pub enum Rating {
    Pass,
    Fail,
    Rating(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rate {
    pub subject: Subject,
    pub rate: Rating,
}

///Actions are things people can do, extensions of sort
/// For example, you could add a verification of the work
/// You could review the task
/// You could grade the job
/// You could submit an estimate or bid
/// Also this should capture edits
/// Ultimately I think I will have this be a set of json-patch documents with a Subject that did it

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub subject: Subject,
    pub patched: Timestamp,
    pub patch: json_patch::Patch,
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use crate::model::task::{BlockedReason, Resolution};

    use super::TaskState;

    #[test]
    fn test_state_serde() {
        let todo_state = TaskState::Todo;
        let blocked_state = TaskState::Blocked(BlockedReason::WaitingOnOther);
        let complete_state = TaskState::Complete(Resolution::Done);

        println!("{}", to_string(&todo_state).unwrap());
        println!("{}", to_string(&blocked_state).unwrap());
        println!("{}", to_string(&complete_state).unwrap());
        let str = "{\"state\":\"todo\"}";
        let json_todo: TaskState = from_str(str).unwrap();
        let TaskState::Todo = json_todo else {
            return assert!(false);
        };
    }
}
