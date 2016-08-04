import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model() {
    const url = `${ENV.apiBaseURL}nag/users`;
    return fetch(url)
      .then(response => response.json())
      .then(({ users }) => ({ users }));
  }
});
