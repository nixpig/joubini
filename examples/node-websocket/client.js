const socket = new WebSocket("ws://localhost/ws");

const message = document.getElementById("message");
const responses = document.getElementById("responses");

const btn = document.getElementById("send-message");
btn.addEventListener("click", (e) => {
  e.preventDefault();
  socket.send(message.value);
});

socket.onmessage = (e) => {
  console.log(e);

  const li = document.createElement("li");

  li.textContent = e.data;
  responses.appendChild(li);
};
