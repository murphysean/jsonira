const uri = 'wss://' + location.host + '/chat';

const getSHA256Hash = async (input) => {
  const textAsBuffer = new TextEncoder().encode(input);
  const hashBuffer = await window.crypto.subtle.digest("SHA-256", textAsBuffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hash = hashArray
    .map((item) => item.toString(16).padStart(2, "0"))
    .join("");
  return hash;
};


function message(data) {
    const line = document.createElement('p');
    line.innerText = data;
    document.getElementById('chat').appendChild(line);
}

function on_load(){
    check_session();
}

async function check_session() {
    //TODO Check if I am currently authenticated
    try{
        const response = await fetch("/session");
        if (response.ok){
            const json = await response.json();
            console.log(json);
            start_chat();
        } else {
            var login_div = document.getElementById("login");
            login_div.style.display = "block";
            throw new Error(`Response status: ${response.status}`);
        }
        
    } catch (error) {
        console.log(error);
    }
    var x = document.getElementById("chat-div");
    x.style.display = "block";
    var x = document.getElementById("chat");
    x.style.display = "block";
}

function start_chat() {
    const ws = new WebSocket(uri);
    ws.onopen = function() {
        document.getElementById('chat-status').innerHTML = '<p><em>Connected!</em></p>';
    };

    ws.onmessage = function(msg) {
        message(msg.data);
    };

    ws.onclose = function() {
        document.getElementById('chat-status').getElementsByTagName('em')[0].innerText = 'Disconnected!';
    };

    send.onclick = function() {
        const msg = document.getElementById('text').value;
        ws.send(msg);
        document.getElementById('text').value = '';

        message('<You>: ' + msg);
    };

    window.ws = ws;
}
