
const title_input = document.getElementById("title");
const reporter_span = document.getElementById("reporter");
const watchers_form = document.getElementById("watchers");
const add_watcher_input = document.getElementById("watcher");
const circle_input = document.getElementById("circle");
const assignee_input = document.getElementById("assignee");
const priority_input = document.getElementById("priority");
const estimate_input = document.getElementById("estimate");
const points_input = document.getElementById("points");
const state_input = document.getElementById("state");
const tags_form = document.getElementById("tags");
const add_tag_input = document.getElementById("tag");
const due_input = document.getElementById("due");
const description_input = document.getElementById("description");
const add_comment_input = document.getElementById("comment");

let authenticated_user = null;
let current_task = null;


//Won't fire because I'm loaded after the page is loaded
function on_load(){
    check_session();
}

function task_from_html(){
    const now = new Date;
    let task = {};

    if (title_input.value){
        task.title = title_input.value;
    }
    if (reporter_span){
        if(reporter_span.dataset){
            task.reporter = {
                id: reporter_span.dataset.id,
                name: reporter_span.dataset.name,
                email: reporter_span.dataset.email,
            };
        }
    }
    let watchers = [];
    for  (child of watchers_form.children){
        if (child.checked){
            let user = get_user_from_dataset(child);
            watchers.push(user);
        }
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
    for (child of tags_form){
        if (child.checked){
            tags.push(child.value);
        }
    }
    task.tags = tags;
    if (created.innerText){
        task.created = now.toISOString();
    }
    if (created.innerText){
        task.updated = now.toISOString();
    }
    if (due_input.value){
        task.due = due_input.value;
    }
    if (description_input.value){
        task.description = description_input.value;
    }
    console.log(task);
    return task;
}

function reset_view(){
    current_task = null;
    title_input.value = "";
    reporter_span.innerText = "";
    watchers_form.replaceChildren();
    circle_input.value = "";
    assignee_input.value = "";
    priority_input.value = "";
    estimate_input.value = "";
    points_input.value = "";
    state_input.value = "";
    //TODO Clear tags
    created.innerText = ""
    updated.innerText = ""
    due_input.value = "";
    description_input.value = "";
    //TODO Clear comments
    add_comment_input.value = "";
}

function update_view(task){
    title_input.value = task.title;
    reporter_span.innerText = task.reporter.name;
    reporter_span.dataset = task.reporter;
    for (w in task.watchers){
        add_watcher(w);
    }
    circle_input.value = task.circle;
    if(task.assignee){
        assignee_input.value = task.assignee.name;
    }
    priority_input.value = task.priority;
    estimate_input.value = task.estimate;
    points_input.value = task.points;
    state_input.value = task.state;
    //TODO Build tags
    created.innerText = task.created;
    updated.innerText = task.updated;
    due_input.value = task.due;
    description_input.value = task.description;
    //TODO Build comments
}

async function check_session() {
    //TODO Check if I am currently authenticated
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

async function get_task(id){
    try{
        const response = await fetch("/api/tasks/" + id)
        if (response.ok){
            const json = await response.json();
            console.log(json);
            current_task = json;
            //TODO call a function that will update the view with the returned task
            update_view(json);
        } else {
            throw new Error(`Response status: ${response.status}`);
        }
    } catch (error){
        console.log(error);
    }
}

//Sends a patch request with the given task document as regular json
async function merge_task(id,task){
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

//Sends a patch request with the given patch document as a patch
async function patch_task(id,patch){
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

async function create_task(){
    const now = new Date;
    let task = task_from_html();
    task.reporter = authenticated_user;

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

function get_user_from_dataset(element) {
    let user = {};
    user.id = Number(element.dataset.id);
    user.name = element.dataset.name;
    user.email = element.dataset.email;
    return user;
} 

function add_watcher_from_input(){
    let v = add_watcher_input.value;
    let option = document.querySelector("#users option[value='" + v + "']");
    if(option){
        add_watcher(option.dataset)
    }else{
        add_watcher({id: null, email: v, name:v})
    }
}

function add_watcher(dataset){
    const eid = watchers.children.length+1;
    let input = document.createElement("input")
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
    let text = document.createTextNode(add_watcher_input.value);
    label.appendChild(text);
    
    watchers_form.appendChild(input);
    watchers_form.appendChild(label);
    add_watcher_input.value = "";
}
document.getElementById("add-watcher-button").addEventListener("click", add_watcher_from_input);

function add_tag(){
    let input = document.createElement("input")
    input.type = "checkbox";
    input.id = "tag-" + add_tag_input.value;
    input.name = "tag";
    input.value = add_tag_input.value;
    input.checked = true;
    let label = document.createElement("label");
    label.for = "tag-" + add_tag_input.value;
    let text = document.createTextNode(add_tag_input.value);
    label.appendChild(text);
    
    tags_form.appendChild(input);
    tags_form.appendChild(label);
    add_tag_input.value = "";
}
document.getElementById("add-tag-button").addEventListener("click", add_tag);

function add_comment(){
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
    const comments = document.getElementById("comments");
    const aid = comments.children.length+1;
    let article = document.createElement("article")
    article.id = "comment-" + aid;
    article.dataset.subject = {
        id: authenticated_user.id,
        name: authenticated_user.name,
        email: authenticated_user.email,
    };
    let div = document.createElement("div");
    let text = document.createTextNode(add_comment_input.value);
    div.appendChild(text);
    article.appendChild(div);
    
    comments.appendChild(article);
    add_comment_input.value = "";
    
    //If I have an active task on the page then send it up as an individual patch
    if(current_task){
        patch_task(current_task.id, [
            {"op": "add", "path": "/comments/-", "value": comment}
        ]);
    }
}
document.getElementById("add-comment-button").addEventListener("click", add_comment);

check_session();