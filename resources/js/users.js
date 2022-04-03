import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";
import {deleteCookie} from "./deleteCookie.js";

function createUsers(users) {
    let album = document.getElementById("users-album")
    album.innerHTML = ``
    for (let user of users) {
        album.innerHTML += `
    <div class="card mb-3">
            <div class="card-body">
                <div class="d-flex flex-column flex-lg-row">
                    <span class="avatar avatar-text rounded-3 me-4 mb-2">${user.name[0].toUpperCase() + user.surname[0].toUpperCase()}</span>
                    <div class="row flex-fill">
                        <div class="col-sm-5">
                            <h4 class="h5 username">${user.name} ${user.surname}</h4>
                            <span class="badge bg-secondary">${user.city !== undefined ? user.city : "Россия"}</span> <span class="badge bg-success">${user.level !== undefined ? user.level : "INTERN"}</span>
                        </div>
                        <div class="col-sm-4 py-2">` + generateComps(user) + `</div>
                        <div class="col-sm-3 text-lg-end">
                            <div class="btn-group-lg">
                                <button type="button" class="btn to-profile" login="${user.login}"><i class="bi bi-eye"></i></button>
                                <button type="button" class="btn" data-bs-toggle="modal" data-bs-target="#mail-modal" ><i class="bi bi-envelope"></i></button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `
    }

    document.querySelectorAll(".to-profile").forEach((button) => {
        button.addEventListener("click", () => {
            let login = button.attributes.getNamedItem("login")
            window.location = "/user/" + login.value
        })
    })


    function profile(login) {
        window.location = "/user/" + login
    }

    function generateComps(user) {
        let output = ''
        for (const comp of user["competences"]) {
            output += `<span class="badge bg-secondary competence">${comp.trim()}</span>`
        }
        return output

    }
}

function generatePagination(page_num, all_pages) {
    let pagination = document.getElementById("pagination-album")
    pagination.innerHTML = `
            <a href="#" id="prev">
                <li><</li>
            </a>`

    let brace = all_pages > 6 ? 6 : all_pages
    for (let i = 1; i <= brace; i++) {
        pagination.innerHTML += `
        <a class = "${i === page_num ? 'is-active' : ''} pg-${i}" href="#">
        <li>${i}</li>
        </a>
        `
    }

    if (all_pages > 6) {
        pagination.innerHTML += ` 
           <a href="#">
                <li>...</li>
            </a>
    `
    }
    pagination.innerHTML += `
            <a href="#" id="next">
                <li>></li>
            </a>
    `
    document.getElementById("prev").addEventListener("click", () => {
        if (page_num > 1) {
            loadPage(page_num - 1)
        }
    })

    document.getElementById("next").addEventListener("click", () => {
        if (page_num < all_pages) {
            loadPage(page_num + 1)
        }
    })

    for (let i = 1; i <= brace; i++) {
        let selector = ".pg-" + i

        document.querySelector(selector).addEventListener("click", () => {
            loadPage(i)
        })
    }


}

function loadPage(page_num) {
    sendRequest("GET", "/api/get_all_users?page=" + page_num, null)
        .then(response => {
            createUsers(response)
        })
        .catch(err => ThrowError(err))

    sendRequest("GET", "/api/get_users_pagination", null)
        .then(response => {
            generatePagination(page_num, response)
        })
        .catch(err => ThrowError(err))


}


document.addEventListener('DOMContentLoaded', () => {
    let users = [{}]
    let pages_count = 0
    let page = 1

    sendRequest("GET", "/api/get_all_users?page=" + page, null)
        .then(response => {
            users = response
            createUsers(users)
        })
        .catch(err => ThrowError(err))

    sendRequest("GET", "/api/get_users_pagination", null)
        .then(response => {
            pages_count = response
            generatePagination(page, pages_count)
        })
        .catch(err => ThrowError(err))


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