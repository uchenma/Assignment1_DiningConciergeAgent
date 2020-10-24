var AWS = require('aws-sdk');
AWS.config.update({region: 'us-east-1'});

exports.handler = async (event) => {
    // TODO implement
    console.log(event);
    
    var id = event.requestContext.identity.cognitoIdentityId;
    
    var event = JSON.parse(event.body);
    
    var lexruntime = new AWS.LexRuntime();

    var lexChatbotParams = {
        botAlias: 'version_two',
        botName: 'DiningConcierge',
        inputText: event.messages[0].unstructured.text,
        userId: id,
        requestAttributes: {},
        sessionAttributes: {}
    };
    
    console.log(lexChatbotParams);
    
    return lexruntime.postText(lexChatbotParams).promise()
    .then((data) =>{
        console.log("data from lex: ", data)
        
        const response = { 
            'headers': {
                "Access-Control-Allow-Headers" : "*",
                "Access-Control-Allow-Origin": "*",
                "Access-Control-Allow-Methods": "*"
    
            }, 
            'statusCode': 200,
            'body': JSON.stringify({
                'messages': [ 
                    {
                        'type': "unstructured", 
                        'unstructured': {
                            'text': data.message
                        }  
                        
                    } 
                ] 
            })
            
        }
        return response;
    })
    .catch((err) =>{
        console.log(err);
    })
    
};
