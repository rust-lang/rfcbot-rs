export default Ember.Helper.helper(function(){
    if(!window.fetch){
        return true;
    }
    return false;
});
