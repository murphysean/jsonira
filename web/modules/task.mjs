export const name = "task";

const title_e = document.getElementById("title");
const title_input = document.getElementById("title-input");
const reporter_e = document.getElementById("reporter");
const watchers_e = document.getElementById("watchers");
const watcher_input = document.getElementById("watcher-input");
const circle_e = document.getElementById("circle");
const circle_input = document.getElementById("circle-input");
const assignee_e = document.getElementById("assignee");
const assignee_input = document.getElementById("assignee-input");
const priority_e = document.getElementById("priority")
const priority_input = document.getElementById("priority-input");
const estimate_e = document.getElementById("estimate");
const estimate_input = document.getElementById("estimate-input");
const points_e = document.getElementById("points");
const points_input = document.getElementById("points-input");
const state_e = document.getElementById("state");
const state_input = document.getElementById("state-input");
const tags_e = document.getElementById("tags");
const tag_input = document.getElementById("tag-input");
const created_e = document.getElementById("created");
const updated_e = document.getElementById("updated");
const due_e = document.getElementById("due");
const due_input = document.getElementById("due-input");
const description_e = document.getElementById("description");
const description_input = document.getElementById("description");
const comments_e = document.getElementById("comments");
const comment_input = document.getElementById("comment-input");

export let authenticated_user = null;
export let current_task = null;

export function set_current_task(task){
    current_task = task;
}
export function set_authenticated_user(user){
    authenticated_user = user;
}

/// Attempts to convert the page with properly tagged elements into a task
export function task_from_inputs(){
    const now = new Date;
    let task = {};

    if (title_input.value){
        task.title = title_input.value;
    }
    let watchers = [];
    for  (child of watchers.children){
        let user = get_user_from_dataset(child);
        watchers.push(user);
    }
    task.watchers = watchers;
    if (circle_input.value){
        task.circle = circle_input.value;
    }
    if(assignee_input.value){
        let v = assignee_input.value;
        let option = document.querySelector("#users option[value='" + v + "']");
        if(option){
            task.assignee = get_user_from_dataset(option);
        }else{
            task.assignee = {id: null, email: v, name:v};
        }
    }
    if (priority_input.value){
        task.priority = priority_input.value;
    }
    if (estimate_input.value){
        task.estimate = estimate_input.value;
    }
    if (points_input.value){
        task.points = Number(points_input.value);
    }
    if (state_input.value){
        let state_option = document.querySelector("#states option[value='" + state_input.value + "']");
        if(state_option){
            task.state = {
                state: state_option.dataset.state,
                reason: state_option.dataset.reason,
                resolution: state_option.dataset.resolution,
            };
        }
    }
    let tags = [];
    for (child of tags_e){
        tags.push(child.dataset.tag);
    }
    task.tags = tags;
    if (due_input.value){
        task.due = due_input.value;
    }
    if (description_input.value){
        task.description = description_input.value;
    }
    console.log(task);
    return task;
}

/// This will reset all inputs to an empty value
export function reset_view(){
    current_task = null;
    title_input.value = "";
    reporter_e.innerText = "";
    watchers_e.replaceChildren();
    circle_input.value = "";
    assignee_input.value = "";
    priority_input.value = "";
    estimate_input.value = "";
    points_input.value = "";
    state_input.value = "";
    //Clear tags
    tags_e.replaceChildren();
    created_e.innerText = ""
    updated_e.innerText = ""
    due_input.value = "";
    description_input.value = "";
    //Clear comments
    comments_e.replaceChildren();
    comment_input.value = "";
}

/// Given a task this will populate all the input values to match
export function update_view(task){
    reset_view();
    title_input.value = task.title;
    reporter_e.dataset.id = task.reporter.id;
    reporter_e.dataset.name = task.reporter.name;
    reporter_e.dataset.email = task.reporter.email;
    reporter_e.innerText = task.reporter.name;
    for (w of task.watchers){
        add_watcher(w);
    }
    if(task.assignee){
        assignee_input.dataset.id = task.assignee.id;
        assignee_input.dataset.name = task.assignee.name;
        assignee_input.dataset.email = task.assignee.email;
        assignee_input.value = task.assignee.name;
    }
    circle_input.value = task.circle;
    priority_input.value = task.priority;
    estimate_input.value = task.estimate;
    points_input.value = task.points;
    state_input.value = task.state.state;
    //Build tags
    for (t of task.tags){
        add_tag(t);
    }
    created_e.datetime = task.created;
    created_e.innerText = task.created;
    updated_e.datetime = task.updated;
    updated_e.innerText = task.updated;
    due_e.datetime = task.due;
    due_input.value = task.due;
    description_input.value = task.description;
    //Build comments
    for (c of task.comments){
        add_comment(c);
    }
}

export async function check_session() {
    //Check if I am currently authenticated
    try{
        const response = await fetch("/session");
        if (response.ok){
            const json = await response.json();
            console.log(json);
            authenticated_user = json;
        } else {
            throw new Error(`Response status: ${response.status}`);
        }
        
    } catch (error) {
        console.log(error);
    }
}

export async function get_task(id){
    try{
        const response = await fetch("/api/tasks/" + id)
        if (response.ok){
            const json = await response.json();
            console.log(json);
            current_task = json;
            //call a function that will update the view with the returned task
            update_view(json);
            //turn on all updates
            for (e of document.getElementsByClassName("update-button")) {e.className = "update-button"}
        } else {
            throw new Error(`Response status: ${response.status}`);
        }
    } catch (error){
        console.log(error);
    }
}

/// Sends a patch request with the given task document as regular json
export async function merge_task(id,task){
    try{
        let request = new Request("/api/tasks/" + id, {
            method: "PATCH",
            headers: { "content-type": "application/json" },
            body: JSON.stringify(task),
        });
        const response = await fetch(request);
        if (response.ok){
            const json = await response.json();
            console.log(json);
            current_task = json;
        } else {
            throw new Error(`Response status: ${response.status}`)
        }
    } catch (error) {
        console.log(error);
    }
}

/// Sends a patch request with the given patch document as a patch
export async function patch_task(id,patch){
    try{
        let request = new Request("/api/tasks/" + id, {
            method: "PATCH",
            headers: { "content-type": "application/json-patch+json" },
            body: JSON.stringify(patch),
        });
        const response = await fetch(request);
        if (response.ok){
            const json = await response.json();
            console.log(json);
            current_task = json;
        } else {
            throw new Error(`Response status: ${response.status}`)
        }
    } catch (error) {
        console.log(error);
    }
}

/// Creates a new task on the server
/// Could be used to clone a task
export async function create_task(task){
    const now = new Date;
    if(!task){
        task = task_from_html();
        task.reporter = authenticated_user;
    }

    console.log(task);

    try{
        let request = new Request("/api/tasks", {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify(task),
        });
        const response = await fetch(request);
        if (response.ok){
            const json = await response.json();
            console.log(json);
            current_task = json;
        } else {
            throw new Error(`Response status: ${response.status}`)
        }
    } catch (error) {
        console.log(error);
    }
}

/// This function will extract a user from any element
/// It uses the dataset api, so those will need to be set with id,name,email
export function get_user_from_dataset(element) {
    let user = {};
    user.id = Number(element.dataset.id);
    user.name = element.dataset.name;
    user.email = element.dataset.email;
    return user;
} 

function update_title_from_input(){
    if(title_input.value){
        //If I have an active task on the page then send it up as an individual patch
        if(current_task){
            current_task.title = title_input.value;
            patch_task(current_task.id, [
                {"op": "replace", "path": "/title", "value": title_input.value}
            ]);
        }
    }
}

function add_watcher_from_input(){
    let v = add_watcher_input.value;
    let option = document.querySelector("#users option[value='" + v + "']");
    if(option){
        add_watcher(option.dataset)
    }else{
        add_watcher({id: null, email: v, name:v})
    }
    add_watcher_input.value = "";
}

function add_watcher(dataset){
    const eid = watchers.children.length+1;
    let input = document.createElement("input");
    input.type = "checkbox";
    input.id = "watcher-" + eid;
    input.name = "watcher";
    input.value = dataset.name;
    if(dataset){
        input.dataset.id = dataset.id;
        input.dataset.name = dataset.name;
        input.dataset.email = dataset.email;
    }
    input.checked = true;
    let label = document.createElement("label");
    label.for = "watcher-" + eid;
    let text = document.createTextNode(dataset.name);
    label.appendChild(text);
    
    watchers_form.appendChild(input);
    watchers_form.appendChild(label);
}

function update_circle_from_input(){
    if(circle_input.value){
        //If I have an active task on the page then send it up as an individual patch
        if(current_task){
            current_task.circle = circle_input.value;
            patch_task(current_task.id, [
                {"op": "replace", "path": "/circle", "value": circle_input.value}
            ]);
        }
    }
}

export function update_assignee_from_input(){
    if(assignee_input.value){
        let assignee = {};
        let v = assignee_input.value;
        if(assignee_input.dataset && assignee_input.dataset.name){
            assignee.id = Number(assignee_input.dataset.id);
            assignee.name = assignee_input.dataset.name;
            assignee.email = assignee_input.dataset.email;
        }else if(document.querySelector("#users option[value='" + v + "']")){
            assignee = get_user_from_dataset(document.querySelector("#users option[value='" + v + "']"));
        }else{
            assignee = {id: null, email: v, name:v};
        }
        //If I have an active task on the page then send it up as an individual patch
        if(current_task){
            current_task.assignee = assignee;
            patch_task(current_task.id, [
                {"op": "replace", "path": "/assignee", "value": assignee}
            ]);
        }
        assignee_e.dataset.id = assignee_input.dataset.id;
        assignee_e.dataset.name = assignee_input.dataset.name;
        assignee_e.dataset.email = assignee_input.dataset.email;
        assignee_e.innerText = assignee.name;
    }
}

export function update_assignee_from_authenticated_user(){
    if(authenticated_user){
        if(current_task){
            current_task.assignee = authenticated_user;
            patch_task(current_task.id, [
                {"op": "replace", "path": "/assignee", "value": authenticated_user}
            ]);
            assignee_e.dataset.id = authenticated_user.id;
            assignee_e.dataset.name = authenticated_user.name;
            assignee_e.dataset.email = authenticated_user.email;
            assignee_e.innerText = authenticated_user.name;
        }
    }
}

function update_priority_from_input(){
    if(priority_input.value){
        //If I have an active task on the page then send it up as an individual patch
        if(current_task){
            current_task.priority = priority_input.value;
            patch_task(current_task.id, [
                {"op": "replace", "path": "/priority", "value": priority_input.value}
            ]);
            //TODO Priority is a bit tricky
            priority_e.innerText = priority_input.value;
        }
    }
}

export function update_estimate_from_input(){
    if(estimate_input.value){
        //If I have an active task on the page then send it up as an individual patch
        if(current_task){
            current_task.estimate = estimate_input.value;
            patch_task(current_task.id, [
                {"op": "replace", "path": "/estimate", "value": estimate_input.value}
            ]);
            estimate_e.innerText = estimate_input.value;
        }
    }
}

export function update_points_from_input(){
    if(points_input.value){
        //If I have an active task on the page then send it up as an individual patch
        if(current_task){
            current_task.points = Number(points_input.value);
            patch_task(current_task.id, [
                {"op": "replace", "path": "/points", "value": Number(points_input.value)}
            ]);
            points_e.innerText = points_input.value;
        }
    }
}

export function update_state_from_input(){
    if(state_input.value){
        let state = {};
        let v = state_input.value;
        let option = document.querySelector("#states option[value='" + v + "']");
        if(option){
            state = {
                state: option.dataset.state,
                reason: option.dataset.reason,
                resolution: option.dataset.resolution,
            };
            //If I have an active task on the page then send it up as an individual patch
            if(current_task){
                current_task.state = state;
                patch_task(current_task.id, [
                    {"op": "replace", "path": "/state", "value": state}
                ]);
                document.getElementById("state").innerText = state.state;
            }
        }
    }
}

function add_tag_from_input(){
    let v = add_tag_input.value;
    //If I have an active task on the page then append this tag as a patch
    if(current_task){
        current_task.tags.push(v);
        patch_task(current_task.id, [
            {"op": "add", "path": "/tags/-", "value": v}
        ]);

    }
    add_tag(v);
    add_tag_input.value = "";
}

function add_tag(tag){
    const eid = tags_form.children.length + 1;
    let input = document.createElement("input")
    input.type = "checkbox";
    input.id = "tag-" + eid;
    input.name = "tag";
    input.value = tag;
    input.checked = true;
    let label = document.createElement("label");
    label.for = "tag-" + eid;
    let text = document.createTextNode(tag);
    label.appendChild(text);
    
    tags_form.appendChild(input);
    tags_form.appendChild(label);
}

function update_due_from_input(){
    if(due_input.value){
        //If I have an active task on the page then send it up as an individual patch
        if(current_task){
            current_task.due = due_input.value;
            patch_task(current_task.id, [
                {"op": "replace", "path": "/due", "value": due_input.value}
            ]);
        }
    }
}

function update_description_from_input(){
    if(description_input.value){
        //If I have an active task on the page then send it up as an individual patch
        if(current_task){
            current_task.due = description_input.value;
            patch_task(current_task.id, [
                {"op": "replace", "path": "/description", "value": description_input.value}
            ]);
        }
    }
}

function add_comment_from_input(){
    const now = new Date;
    let comment = {
        subject: {
            id: authenticated_user.id,
            name: authenticated_user.name,
            email: authenticated_user.email,
        },
        comment: add_comment_input.value,
        content_type: "text/plain",
        created: now.toISOString(),
    };

    add_comment(comment);

    //If I have an active task on the page then send it up as an individual patch
    if(current_task){
        patch_task(current_task.id, [
            {"op": "add", "path": "/comments/-", "value": comment}
        ]);
    }
}

function add_comment(comment){
    const eid = comments_section.children.length+1;
    let article = document.createElement("article")
    article.id = "comment-" + eid;
    article.dataset.subject = {
        id: authenticated_user.id,
        name: authenticated_user.name,
        email: authenticated_user.email,
    };
    let footer = document.createElement("footer");
    footer.innerText = "By " + comment.subject.name + " on " + comment.created;
    let div = document.createElement("div");
    let text = document.createTextNode(comment.comment);
    div.appendChild(text);
    article.appendChild(div);
    article.appendChild(footer);
    
    comments.appendChild(article);
    add_comment_input.value = "";
    
}

//modifies _date_
function setToMonday( date ) {
    var day = date.getDay() || 7;  
    if( day !== 1 ) 
        date.setHours(-24 * (day - 1)); 
    return date;
}

function to_local_date_string( date ) {
    return date.getYear() + "-"
}

function parse_file_string(file_as_string){
    //let's start assuming it's json
    let obj = JSON.parse(file_as_string);

    //Get now
    let now = new Date();
    let day_of_week = setToMonday(new Date());


    let day_plus = day_of_week.getDate();
    //MTWThFSaSu
    for (i=0;i<7;i++){
        day_of_week.setDate(day_plus);
        let day_of_week_string = day_of_week.toLocaleDateString('en-CA', {year: 'numeric', month: '2-digit', day: '2-digit'});
        //Create an unassigned task
        for (dt of obj.daily){
            dt.circle = obj.circle;
            dt.due = day_of_week_string;
            dt.tags = [];
            dt.tags = dt.tags = dt.tags.concat(obj.tags, ["daily"])
            console.log("Creating", dt);
            create_task(dt);
        }
        for (dt of obj.daily_parents){
            for (p of obj.family.parents){
                dt.circle = obj.circle;
                dt.assignee = p;
                dt.due = day_of_week_string;
                dt.tags = [];
                dt.tags = dt.tags = dt.tags.concat(obj.tags, ["daily", "daily-parent"])
                console.log("Creating", dt);
                create_task(dt);
            }
        }
        for (dt of obj.daily_children){
            for (c of obj.family.children){
                dt.circle = obj.circle;
                dt.assignee = c;
                dt.due = day_of_week_string;
                dt.tags = [];
                dt.tags = dt.tags = dt.tags.concat(obj.tags, ["daily", "daily-child"])
                console.log("Creating", dt);
                create_task(dt);
            }
        }
        day_plus = day_plus + 1;
    }
    //Set day back to Sunday
    day_of_week.setDate(-1);
    let day_of_week_string = day_of_week.toLocaleDateString('en-CA', {year: 'numeric', month: '2-digit', day: '2-digit'});
    for (dt of obj.weekly){
        dt.circle = obj.circle;
        dt.due = day_of_week_string;
        dt.tags = [];
        dt.tags = dt.tags = dt.tags.concat(obj.tags, ["weekly"])
        console.log("Creating", dt);
        create_task(dt);
    }
    for (dt of obj.weekly_parents){
        for (p of obj.family.parents){
            dt.circle = obj.circle;
            dt.assignee = p;
            dt.due = day_of_week_string;
            dt.tags = [];
            dt.tags = dt.tags = dt.tags.concat(obj.tags, ["weekly", "weekly-parent"])
            console.log("Creating", dt);
            create_task(dt);
        }
    }
    for (dt of obj.weekly_children){
        for (c of obj.family.children){
            dt.circle = obj.circle;
            dt.assignee = c;
            dt.due = day_of_week_string;
            dt.tags = [];
            dt.tags = dt.tags = dt.tags.concat(obj.tags, ["weekly", "weekly-child"])
            console.log("Creating", dt);
            create_task(dt);
        }
    }
    
}

function load_script(){
    const gsi = document.getElementById("get-script-input");
    const file_name = gsi.value;
    const reader = new FileReader();
    const file = gsi.files[0];

    reader.addEventListener(
    "load",
        () => {
            // this will then display a text file
            parse_file_string(reader.result);
        },
        false,
    );
    if(file){
        reader.readAsText(file);
    }
}

check_session();