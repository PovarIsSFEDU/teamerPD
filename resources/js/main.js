import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";

document.addEventListener('DOMContentLoaded', () => {
    //Методы отправки данных для авторизации
    let _button_signout = document.querySelector(".signout");

    _button_signout.addEventListener("click", () => {
        sendRequest("GET", "/logout", null)
            .catch(err => ThrowError(err))
        deleteCookie("Authenticate")
        window.location.replace("/login")
    })

    function deleteCookie(name) {
        document.cookie = encodeURIComponent(name) + "=; max-age=0";
    }

});