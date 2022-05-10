import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";

let levels = ["Urgent", "Normal", "Secondary", "Minor"]

let task_level = document.getElementById("task_lvl")

for (const level of levels) {
    let button = document.getElementById(level)
    button.addEventListener("click", () => {
        task_level.innerHTML = button.innerHTML + " " + `<i class="uil uil-arrow-down sp-uil"></i>`

    })
}

let create = document.getElementById("savetask")
create.addEventListener("click", () => {
    let data = {
        name: document.getElementById("taskname").value,
        description: document.getElementById("task_description").value,
        status: "Open",
        assignee: document.getElementById("user_for_task").value,
        priority: "Urgent",
        expiration: "21.05.2022"
    }
    sendRequest("POST", "/api/create_task", data)
        .then(response => console.log(response))
        .catch(err => ThrowError(err))
})