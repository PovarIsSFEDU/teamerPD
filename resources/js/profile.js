import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";
import {deleteCookie} from "./deleteCookie.js";

var competences = []
// Load competences from db

document.addEventListener('DOMContentLoaded', () => {
    const _button_signout = document.querySelector(".signout");
    _button_signout.addEventListener("click", () => {
        sendRequest("GET", "/logout", null)
            .catch(err => ThrowError(err))
        deleteCookie("Authenticate")
        window.location.replace("/login")
    })

    const _button_competence_add = document.getElementById("competence-add");
    const _div_competences = document.getElementById("competences");
    const _input_competence = document.getElementById("competence")

    function _input_method(e) {
        if(e.keyCode == 13) {
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
    
    function remove_competence(i) {
        competences.splice(i, 1)
        update_compentences()
    }
    
    function add_competence(s) {
        if (competences.findIndex((v) => {return v == s}) != -1) return
        competences.push(s)
        update_compentences()
    }
});