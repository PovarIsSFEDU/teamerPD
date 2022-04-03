import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";
import {deleteCookie} from "./deleteCookie.js";

var recommended_competences = []

var competences = []

let users = [{}]

function fetchUsers() {
    sendRequest("GET", "/api/get_all_users?page=0", null)
        .then(response => {
            users = response
        })
        .catch(err => ThrowError(err))
}

// Load competences from db

document.addEventListener('DOMContentLoaded', () => {
    const _button_signout = document.querySelector(".signout");
    _button_signout.addEventListener("click", () => {
        sendRequest("GET", "/logout", null)
            .catch(err => ThrowError(err))
        deleteCookie("Authenticate")
        window.location.replace("/login")
    })


    const _buttonSave = document.querySelector("#saveTeam");


    _buttonSave.addEventListener("click", () => {
        let data = {
            name: document.querySelector("#teamname").value,
            logo: null,
            captain: "",
            members: [],
            short_bio: document.querySelector("#short_description").value,
            long_bio: document.querySelector("#long_description").value,
            competences: competences
        }
        sendRequest("POST", "/api/create_team?team_name=" + document.querySelector("#teamname").value, data)
            .then(a => {
                window.location = "/teams"
            })
            .catch(err => ThrowError(err))

    })


    const _auto_competence = document.getElementById("competence-auto")
    const _button_competence_add = document.getElementById("competence-add")
    const _div_competences = document.getElementById("competences")
    const _input_competence = document.getElementById("competence")


    function check_autocomplete(cmp, inp) {
        return cmp.toLowerCase().indexOf(inp.toLowerCase()) != -1
    }

    function update_autocomplete() {
        let input = _input_competence.value
        _auto_competence.innerHTML = ""
        recommended_competences.forEach((element, index) => {
            if (!check_autocomplete(element, input)) return
            _auto_competence.innerHTML += '<li><button class="dropdown-item" id="auto-competence-' + index + '">' + element + '</button></li>'
        });
        recommended_competences.forEach((element, index) => {
            if (!check_autocomplete(element, input)) return
            let htmlelem = document.getElementById('auto-competence-' + index)
            htmlelem.addEventListener("click", () => {
                _dropdown_method(htmlelem)
            })
        });
    }

    update_autocomplete()

    function _dropdown_method(e) {
        add_competence(e.textContent)
    }

    _input_competence.addEventListener("input", update_autocomplete)


    function _input_method(e) {
        if (e.keyCode == 13) {
            _button_method()
        }
    }

    function _button_method() {
        let competence = _input_competence.value
        if (competence == "") return
        _input_competence.value = ""
        add_competence(competence)
    }

    _button_competence_add.addEventListener("click", _button_method)
    _input_competence.addEventListener("keydown", _input_method)

    function update_compentences() {
        _div_competences.innerHTML = ""
        competences.forEach((element, index) => {
            _div_competences.innerHTML += '<h5><span class="badge bg-competence">' + element + '<button class="remove-competence" id="competence-' + index + '"></button></span></h5>'
        });
        competences.forEach((element, index) => {
            let htmlelem = document.getElementById('competence-' + index)
            htmlelem.addEventListener("click", () => {
                remove_competence(index)
            })
        });
    }

    let _rmComp_first = document.querySelectorAll(".remove-competence")
    if (_rmComp_first != null) {
        _rmComp_first.forEach((elem) => {
            competences.push(elem.parentNode.firstChild.textContent)
            elem.addEventListener("click", () => {
                elem.parentNode.parentNode.removeChild(elem.parentNode);
                competences.splice(competences.indexOf(elem.parentNode.firstChild.textContent), 1)
            })
        });
    }

    function remove_competence(i) {
        competences.splice(i, 1)
        update_compentences()
    }

    function add_competence(s) {
        if (competences.findIndex((v) => {
            return v == s
        }) != -1) return
        competences.push(s)
        update_compentences()
    }

    _input_competence.addEventListener('focus', (event) => {
        _auto_competence.style.display = "block"
        update_autocomplete()
    });

    _input_competence.addEventListener('focusout', (event) => {
        setTimeout(() => {
            _auto_competence.style.display = "none";
        }, 100)
    });
});


// Strashno ochen
recommended_competences = [
    "Web-разработка",
    "Разработка микросервисов",
    "Аналитика и системный анализ",
    "Big Data Research",
    "Bots",
    "AI development"
]