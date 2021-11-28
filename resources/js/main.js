import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";
import {deleteCookie} from "./deleteCookie.js";

document.addEventListener('DOMContentLoaded', () => {
    //Методы отправки данных для авторизации
    const _button_signout = document.querySelector(".signout");

    if (_button_signout != null) {
        _button_signout.addEventListener("click", () => {
            sendRequest("GET", "/logout", null)
                .then(redir())
                .catch(err => ThrowError(err))


        })
    }

    function redir() {
        deleteCookie("Authenticate")
        window.location.replace("/login")
    }


});