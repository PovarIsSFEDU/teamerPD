document.addEventListener('DOMContentLoaded', () => {
    //Методы отправки данных для авторизации
    let _button_signout = document.querySelector(".signout");

    _button_signout.addEventListener("click", () => {
        sendRequest("GET", "/logout", null)
            .catch(err => ThrowError(err))
        deleteCookie("Authenticate")
        window.location.replace("/login")
    })


    function ThrowError(data) {
        console.log(data)
    }


    function sendRequest(method, url, json) {
        return new Promise((resolve, reject) => {
            const xhr = new XMLHttpRequest()
            xhr.open(method, url)
            xhr.responseType = 'json'
            xhr.setRequestHeader('Content-type', 'application/json')
            xhr.onload = () => {
                if (xhr.status >= 400) {
                    reject(xhr.response)
                } else {
                    resolve(xhr.response)
                }
            }
            xhr.onerror = () => {
                console.log(onerror)
            }
            xhr.send(JSON.stringify(json))
        })
    }


    function deleteCookie(name) {
        document.cookie = encodeURIComponent(name) + "=; max-age=0";
    }

});