import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";
import {deleteCookie} from "./deleteCookie.js";

var teaminfo = document.getElementById('teaminfo')

teaminfo.addEventListener('show.bs.modal', (event) => {
    var button = event.relatedTarget
    var teamname = button.getAttribute('data-bs-teamname')
    var modalTitle = teaminfo.querySelector('.modal-title')
    var modalBody = teaminfo.querySelector('.modal-body')

    modalTitle.textContent = teamname
    //modalBody.value = recipient

    const _button_signout = document.querySelector(".signout");

    if (_button_signout != null) {
        _button_signout.addEventListener("click", () => {
            sendRequest("GET", "/logout", null)
                .catch(err => ThrowError(err))
            deleteCookie("Authenticate")
            window.location.replace("/login")
        })
    }
})