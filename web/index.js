function on_load(){
    check_session();
}

async function check_session() {
    //Check if I am currently authenticated
    try{
        const response = await fetch("/session");
        if (response.ok){
            const json = await response.json();
            console.log(json);
        } else {
            var login_div = document.getElementById("login");
            login_div.style.display = "block";
            throw new Error(`Response status: ${response.status}`);
        }
        
    } catch (error) {
        console.log(error);
    }
}