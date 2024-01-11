const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });

  document.getElementById("myTextArea").value += "Something cool\n";
}


function handleCheckboxChange() {
  document.getElementById("myTextArea").value += "Something cool\n";
  var checkbox = document.getElementById('cbHTTP');
  invoke('rb_toggled', { v: checkbox })
}

// Wait for the document to be ready before running JavaScript/jQuery
    $(document).ready(function () {
      // Initialize Semantic UI checkbox
      $('.ui.checkbox').checkbox();

      // Handle checkbox change event
      $('.ui.checkbox').change(function () {
        if ($(this).checkbox('is checked')) {
          console.log("Checkbox is checked");
          document.getElementById("myTextArea").value += "Something cool\n";

          // Perform actions or call other functions when the checkbox is checked
        } else {
          console.log("Checkbox is unchecked");
          // Perform actions or call other functions when the checkbox is unchecked
        }
      });
    });

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
