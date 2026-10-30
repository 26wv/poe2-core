'use strict';
const native = require('./index.js');
// Yields the lines available now; iteration ends when caught up, so iterate again later for more (ADR 0017).
native.LogTail.prototype[Symbol.iterator] = function* () { for (let l = this.nextLine(); l !== null; l = this.nextLine()) yield l; };
module.exports = native;
