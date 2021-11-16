export function addCookie(name, value) {
    let _expires = new Date(Date.now() + 86400e3 * 0.5);
    document.cookie = encodeURIComponent(name) + '=' + encodeURIComponent(value) + "; expires=" + _expires;
}