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

function normalize(value) {
    if (typeof value !== undefined && value !== "") {
        let arr = value.split("-")
        return arr[2] + "." + arr[1] + "." + arr[0]
    } else {
        ThrowError("Choose date!");
    }
}

function prior(raw_priority) {
    switch (raw_priority) {
        case "Срочная" : {
            return "Urgent"
        }
        case "Обычная" : {
            return "Normal"
        }
        case "Побочная" : {
            return "Secondary"
        }
        case "Незначительная" : {
            return "Minor"
        }
    }
}

create.addEventListener("click", () => {
    let assignee = document.getElementById("user_for_task").value
    let raw_priority = prior(document.getElementById("task_lvl").textContent.trim())

    if (typeof assignee !== undefined && assignee !== "") {
        let data = {
            name: document.getElementById("taskname").value,
            description: document.getElementById("task_description").value,
            status: "InProgress",
            assignee: assignee,
            priority: raw_priority,
            expiration: normalize(document.getElementById("date_for_task").value)
        }
        sendRequest("POST", "/api/create_task", data)
            .then(response => window.location.reload())
            .catch(err => ThrowError(err))
    } else {
        ThrowError("Choose assignee!")
    }
})

let removment = document.querySelectorAll(".removement")
removment.forEach(value => {
    value.addEventListener("click", () => {
        let login = value.attributes.getNamedItem("login")
        console.log(login.value)
        sendRequest("GET", "/api/remove_from_team?user=" + login.value, null)
            .then(response => {
                console.log(response)
                window.location.reload();
            })
            .catch(err => ThrowError(err))
    })
})